use axum::{
    extract::{Json, Path, Query, Request, State},
    http::{header::HeaderName, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use surgedb_core::filter::Filter;
use surgedb_core::{Config as DbConfig, Database, DistanceMetric, QuantizationType};
use sysinfo::System;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};
use utoipa::{IntoParams, OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(RustEmbed)]
#[folder = "dist/"]
struct Assets;

async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    let path = if path.is_empty() || path == "/" {
        "index.html".to_string()
    } else {
        path
    };

    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            )
                .into_response()
        }
        None => {
            // Fallback to index.html for SPA routing
            if let Some(content) = Assets::get("index.html") {
                (
                    [(axum::http::header::CONTENT_TYPE, "text/html")],
                    content.data,
                )
                    .into_response()
            } else {
                (StatusCode::NOT_FOUND, "Not Found").into_response()
            }
        }
    }
}

async fn index_handler() -> impl IntoResponse {
    static_handler(Path("index.html".to_string())).await
}

// =============================================================================
// Configuration
// =============================================================================

#[derive(Clone)]
struct AppConfig {
    port: u16,
    web_port: u16,
    api_key: Option<String>,
    log_level: String,
    cors_allow_origin: String,
    request_timeout_secs: u64,
    max_request_size_bytes: usize,
    data_dir: String,
}

impl AppConfig {
    fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            web_port: std::env::var("WEB_PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .unwrap_or(3001),
            api_key: std::env::var("API_KEY").ok(),
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            cors_allow_origin: std::env::var("CORS_ALLOW_ORIGIN")
                .unwrap_or_else(|_| "*".to_string()),
            request_timeout_secs: std::env::var("REQUEST_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            max_request_size_bytes: std::env::var("MAX_REQUEST_SIZE_BYTES")
                .unwrap_or_else(|_| "10485760".to_string()) // 10MB
                .parse()
                .unwrap_or(10 * 1024 * 1024),
            data_dir: std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string()),
        }
    }
}

use chrono::{DateTime, Utc};
use parking_lot::RwLock as PRwLock;
use std::collections::VecDeque;

// =============================================================================
// Configuration
// =============================================================================

#[derive(Serialize, Clone, Debug, ToSchema)]
struct MetricsSnapshot {
    timestamp: DateTime<Utc>,
    memory_usage_mb: u64,
    read_requests: u64,
    write_requests: u64,
    avg_latency_ms: f64,
    storage_usage_bytes: u64,
}

struct MetricsRegistry {
    history: PRwLock<VecDeque<MetricsSnapshot>>,
    current_reads: std::sync::atomic::AtomicU64,
    current_writes: std::sync::atomic::AtomicU64,
    total_latency_us: std::sync::atomic::AtomicU64,
    latency_count: std::sync::atomic::AtomicU64,
}

impl MetricsRegistry {
    fn new() -> Self {
        Self {
            history: PRwLock::new(VecDeque::with_capacity(600)),
            current_reads: std::sync::atomic::AtomicU64::new(0),
            current_writes: std::sync::atomic::AtomicU64::new(0),
            total_latency_us: std::sync::atomic::AtomicU64::new(0),
            latency_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    fn record_request(&self, method: &Method, latency_ms: f64) {
        match *method {
            Method::GET => {
                self.current_reads
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            Method::POST | Method::PUT | Method::DELETE | Method::PATCH => {
                self.current_writes
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            _ => {}
        }
        let latency_us = (latency_ms * 1000.0) as u64;
        self.total_latency_us
            .fetch_add(latency_us, std::sync::atomic::Ordering::Relaxed);
        self.latency_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
    config: AppConfig,
    start_time: Instant,
    metrics: Arc<MetricsRegistry>,
}

#[derive(Deserialize, ToSchema)]
struct CreateCollectionRequest {
    #[schema(example = "my_collection")]
    name: String,
    #[schema(example = 384)]
    dimensions: usize,
    #[serde(default)]
    #[schema(example = "Cosine")]
    distance_metric: DistanceMetric,
    #[serde(default)]
    quantization: Option<QuantizationType>,
}

#[derive(Deserialize, ToSchema)]
struct InsertRequest {
    #[schema(example = "vec1")]
    id: String,
    #[schema(example = "[0.1, 0.2, 0.3]")]
    vector: Vec<f32>,
    metadata: Option<Value>,
}

#[derive(Deserialize, ToSchema)]
struct BatchInsertRequest {
    vectors: Vec<InsertRequest>,
}

#[derive(Deserialize, ToSchema)]
struct SearchRequest {
    #[schema(example = "[0.1, 0.2, 0.3]")]
    vector: Vec<f32>,
    #[schema(example = 10)]
    k: usize,
    filter: Option<Filter>,
}

#[derive(Serialize, ToSchema)]
struct SearchResult {
    id: String,
    distance: f32,
    metadata: Option<Value>,
}

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize, ToSchema)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
    memory_usage_mb: u64,
}

#[derive(Serialize, ToSchema)]
struct StatsResponse {
    uptime_seconds: u64,
    database: surgedb_core::DatabaseStats,
}

#[derive(Deserialize, IntoParams)]
struct PaginationParams {
    #[param(example = 0)]
    offset: Option<usize>,
    #[param(example = 10)]
    limit: Option<usize>,
}

#[derive(Serialize, ToSchema)]
struct VectorResponse {
    id: String,
    vector: Vec<f32>,
    metadata: Option<Value>,
}

// =============================================================================
// OpenAPI Documentation
// =============================================================================

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        get_stats,
        get_metrics_history,
        create_collection,
        list_collections,
        delete_collection,
        insert_vector,
        list_vectors,
        batch_insert_vector,
        upsert_vector,
        get_vector,
        delete_vector,
        search_vector,
    ),
    components(
        schemas(
            CreateCollectionRequest, InsertRequest, BatchInsertRequest,
            SearchRequest, SearchResult, ErrorResponse, HealthResponse,
            StatsResponse, VectorResponse, MetricsSnapshot, VectorListEntry
        )
    ),
    tags(
        (name = "surgedb", description = "SurgeDB Vector Search API")
    )
)]
struct ApiDoc;

// =============================================================================
// Middleware
// =============================================================================

async fn metrics_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    let start = Instant::now();
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64() * 1000.0;
    state.metrics.record_request(&method, latency);

    response
}

async fn auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    if let Some(expected_key) = &state.config.api_key {
        let auth_header = req.headers().get("x-api-key").and_then(|v| v.to_str().ok());

        if auth_header != Some(expected_key) {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid or missing API key".to_string(),
                }),
            ));
        }
    }
    Ok(next.run(req).await)
}

// =============================================================================
// Main Entry Point
// =============================================================================

#[tokio::main]
async fn main() {
    let config = AppConfig::from_env();

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    fmt().with_env_filter(env_filter).with_target(false).init();

    info!("Starting SurgeDB Server v{}", env!("CARGO_PKG_VERSION"));

    let db = Database::open(&config.data_dir).expect("Failed to open database");
    let metrics = Arc::new(MetricsRegistry::new());
    let state = AppState {
        db: Arc::new(db),
        config: config.clone(),
        start_time: Instant::now(),
        metrics: metrics.clone(),
    };

    // Background task for metrics collection
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut sys = System::new_all();
        // Initial snapshot
        {
            sys.refresh_all();
            let pid = sysinfo::get_current_pid().ok();
            let process_memory = pid
                .and_then(|p| sys.process(p))
                .map(|p| p.memory())
                .unwrap_or(0);
            let db_stats = state_clone.db.get_stats();
            let snapshot = MetricsSnapshot {
                timestamp: Utc::now(),
                memory_usage_mb: process_memory / 1024 / 1024,
                read_requests: 0,
                write_requests: 0,
                avg_latency_ms: 0.0,
                storage_usage_bytes: db_stats.total_memory_bytes as u64,
            };
            state_clone.metrics.history.write().push_back(snapshot);
        }

        loop {
            tokio::time::sleep(Duration::from_secs(6)).await;
            sys.refresh_all();

            let pid = sysinfo::get_current_pid().ok();
            let process_memory = pid
                .and_then(|p| sys.process(p))
                .map(|p| p.memory())
                .unwrap_or(0);

            let reads = state_clone
                .metrics
                .current_reads
                .swap(0, std::sync::atomic::Ordering::Relaxed);
            let writes = state_clone
                .metrics
                .current_writes
                .swap(0, std::sync::atomic::Ordering::Relaxed);
            let count = state_clone
                .metrics
                .latency_count
                .swap(0, std::sync::atomic::Ordering::Relaxed);
            let total_lat_us = state_clone
                .metrics
                .total_latency_us
                .swap(0, std::sync::atomic::Ordering::Relaxed);

            let avg_latency = if count > 0 {
                (total_lat_us as f64 / 1000.0) / count as f64
            } else {
                0.0
            };

            // Storage usage calculation
            let db_stats = state_clone.db.get_stats();
            let storage_bytes = db_stats.total_memory_bytes as u64;

            let snapshot = MetricsSnapshot {
                timestamp: Utc::now(),
                memory_usage_mb: process_memory / 1024 / 1024,
                read_requests: reads,
                write_requests: writes,
                avg_latency_ms: avg_latency,
                storage_usage_bytes: storage_bytes,
            };

            let mut history = state_clone.metrics.history.write();
            if history.len() >= 600 {
                history.pop_front();
            }
            history.push_back(snapshot);
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(config.cors_allow_origin.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            HeaderName::from_static("x-api-key"),
        ]);

    let api_routes = Router::new()
        .route("/stats", get(get_stats))
        .route("/metrics/history", get(get_metrics_history))
        .route(
            "/collections",
            post(create_collection).get(list_collections),
        )
        .route("/collections/:name", delete(delete_collection))
        .route(
            "/collections/:name/vectors",
            post(insert_vector).get(list_vectors),
        )
        .route(
            "/collections/:name/vectors/batch",
            post(batch_insert_vector),
        )
        .route("/collections/:name/upsert", post(upsert_vector))
        .route(
            "/collections/:name/vectors/:id",
            get(get_vector).delete(delete_vector),
        )
        .route("/collections/:name/search", post(search_vector))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let api_router = Router::new()
        .route("/health", get(health_check))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            metrics_middleware,
        ))
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(
            config.request_timeout_secs,
        )))
        .layer(RequestBodyLimitLayer::new(config.max_request_size_bytes))
        .layer(cors);

    let api_app = api_router.clone().with_state(state.clone());

    let web_app = Router::new()
        .nest("/api", api_router)
        .route("/", get(index_handler))
        .route("/*path", get(static_handler))
        .fallback(index_handler)
        .with_state(state);

    let api_addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let web_addr = SocketAddr::from(([0, 0, 0, 0], config.web_port));

    info!("API Server listening on {}", api_addr);
    info!("Web Interface listening on {}", web_addr);

    let api_listener = tokio::net::TcpListener::bind(api_addr).await.unwrap();
    let web_listener = tokio::net::TcpListener::bind(web_addr).await.unwrap();

    let api_server = axum::serve(api_listener, api_app).with_graceful_shutdown(shutdown_signal());
    let web_server = axum::serve(web_listener, web_app).with_graceful_shutdown(shutdown_signal());

    tokio::select! {
        res = api_server => {
            if let Err(e) = res {
                warn!("API server error: {}", e);
            }
        }
        res = web_server => {
            if let Err(e) = res {
                warn!("Web server error: {}", e);
            }
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C, shutting down..."),
        _ = terminate => info!("Received SIGTERM, shutting down..."),
    }
}

// =============================================================================
// Route Handlers
// =============================================================================

#[utoipa::path(
    get,
    path = "/metrics/history",
    responses(
        (status = 200, description = "Metrics history", body = [MetricsSnapshot])
    ),
    security(("api_key" = []))
)]
async fn get_metrics_history(State(state): State<AppState>) -> Json<Vec<MetricsSnapshot>> {
    let history = state.metrics.history.read();
    Json(history.iter().cloned().collect())
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Server is healthy", body = HealthResponse)
    )
)]
async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    // Only refresh process info, not entire system
    let mut sys = System::new();
    let pid = sysinfo::get_current_pid().ok();
    if let Some(p) = pid {
        sys.refresh_process(p);
    }

    let process_memory = pid
        .and_then(|p| sys.process(p))
        .map(|p| p.memory())
        .unwrap_or(0);

    Json(HealthResponse {
        status: "OK".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        memory_usage_mb: process_memory / 1024 / 1024,
    })
}

#[utoipa::path(
    get,
    path = "/stats",
    responses(
        (status = 200, description = "Database statistics", body = StatsResponse)
    ),
    security(("api_key" = []))
)]
async fn get_stats(State(state): State<AppState>) -> Json<StatsResponse> {
    let stats = state.db.get_stats();
    let uptime = state.start_time.elapsed().as_secs();
    Json(StatsResponse {
        uptime_seconds: uptime,
        database: stats,
    })
}

#[utoipa::path(
    post,
    path = "/collections",
    request_body = CreateCollectionRequest,
    responses(
        (status = 200, description = "Collection created"),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    ),
    security(("api_key" = []))
)]
async fn create_collection(
    State(state): State<AppState>,
    Json(payload): Json<CreateCollectionRequest>,
) -> Result<&'static str, (StatusCode, Json<ErrorResponse>)> {
    let config = DbConfig {
        dimensions: payload.dimensions,
        distance_metric: payload.distance_metric,
        quantization: payload.quantization.unwrap_or(QuantizationType::None),
        ..DbConfig::default()
    };

    match state.db.create_collection(&payload.name, config) {
        Ok(_) => {
            info!("Created collection: {}", payload.name);
            Ok("Created")
        }
        Err(e) => {
            warn!("Failed to create collection {}: {}", payload.name, e);
            Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "/collections",
    responses(
        (status = 200, description = "List of collection names", body = [String])
    ),
    security(("api_key" = []))
)]
async fn list_collections(State(state): State<AppState>) -> Json<Vec<String>> {
    Json(state.db.list_collections())
}

#[utoipa::path(
    delete,
    path = "/collections/{name}",
    params(
        ("name" = String, Path, description = "Collection name")
    ),
    responses(
        (status = 200, description = "Collection deleted"),
        (status = 404, description = "Collection not found", body = ErrorResponse)
    ),
    security(("api_key" = []))
)]
async fn delete_collection(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<&'static str, (StatusCode, Json<ErrorResponse>)> {
    match state.db.delete_collection(&name) {
        Ok(_) => {
            info!("Deleted collection: {}", name);
            Ok("Deleted")
        }
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[utoipa::path(
    post,
    path = "/collections/{name}/vectors",
    params(
        ("name" = String, Path, description = "Collection name")
    ),
    request_body = InsertRequest,
    responses(
        (status = 200, description = "Vector inserted"),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 404, description = "Collection not found", body = ErrorResponse)
    ),
    security(("api_key" = []))
)]
async fn insert_vector(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<InsertRequest>,
) -> Result<&'static str, (StatusCode, Json<ErrorResponse>)> {
    let collection = state.db.get_collection(&name).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let result = tokio::task::spawn_blocking(move || {
        collection.insert(payload.id, &payload.vector, payload.metadata)
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    match result {
        Ok(_) => Ok("Inserted"),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[utoipa::path(
    post,
    path = "/collections/{name}/upsert",
    params(
        ("name" = String, Path, description = "Collection name")
    ),
    request_body = InsertRequest,
    responses(
        (status = 200, description = "Vector upserted"),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    ),
    security(("api_key" = []))
)]
async fn upsert_vector(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<InsertRequest>,
) -> Result<&'static str, (StatusCode, Json<ErrorResponse>)> {
    let collection = state.db.get_collection(&name).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let result = tokio::task::spawn_blocking(move || {
        collection.upsert(payload.id, &payload.vector, payload.metadata)
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    match result {
        Ok(_) => Ok("Upserted"),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[utoipa::path(
    post,
    path = "/collections/{name}/vectors/batch",
    params(
        ("name" = String, Path, description = "Collection name")
    ),
    request_body = BatchInsertRequest,
    responses(
        (status = 200, description = "Number of vectors upserted", body = usize),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    ),
    security(("api_key" = []))
)]
async fn batch_insert_vector(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<BatchInsertRequest>,
) -> Result<Json<usize>, (StatusCode, Json<ErrorResponse>)> {
    let collection = state.db.get_collection(&name).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let count = payload.vectors.len();
    let result = tokio::task::spawn_blocking(move || {
        let items: Vec<(String, Vec<f32>, Option<Value>)> = payload
            .vectors
            .into_iter()
            .map(|item| (item.id, item.vector, item.metadata))
            .collect();

        collection.upsert_batch(items)?;
        Ok::<(), surgedb_core::Error>(())
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    match result {
        Ok(_) => Ok(Json(count)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[utoipa::path(
    get,
    path = "/collections/{name}/vectors/{id}",
    params(
        ("name" = String, Path, description = "Collection name"),
        ("id" = String, Path, description = "Vector ID")
    ),
    responses(
        (status = 200, description = "Vector found", body = VectorResponse),
        (status = 404, description = "Vector not found", body = ErrorResponse)
    ),
    security(("api_key" = []))
)]
async fn get_vector(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<Json<VectorResponse>, (StatusCode, Json<ErrorResponse>)> {
    let collection = state.db.get_collection(&name).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let id_clone = id.clone();
    let result = tokio::task::spawn_blocking(move || collection.get(&id_clone))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    match result {
        Ok(Some((vector, metadata))) => Ok(Json(VectorResponse {
            id,
            vector,
            metadata,
        })),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Vector not found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[utoipa::path(
    delete,
    path = "/collections/{name}/vectors/{id}",
    params(
        ("name" = String, Path, description = "Collection name"),
        ("id" = String, Path, description = "Vector ID")
    ),
    responses(
        (status = 200, description = "Vector deleted"),
        (status = 404, description = "Vector not found", body = ErrorResponse)
    ),
    security(("api_key" = []))
)]
async fn delete_vector(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<&'static str, (StatusCode, Json<ErrorResponse>)> {
    let collection = state.db.get_collection(&name).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let id_clone = id.clone();
    let result = tokio::task::spawn_blocking(move || collection.delete(&id_clone))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    match result {
        Ok(true) => Ok("Deleted"),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Vector not found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Serialize, ToSchema)]
struct VectorListEntry {
    id: String,
    metadata: Option<Value>,
}

#[utoipa::path(
    get,
    path = "/collections/{name}/vectors",
    params(
        ("name" = String, Path, description = "Collection name"),
        PaginationParams
    ),
    responses(
        (status = 200, description = "List of vector records", body = [VectorListEntry])
    ),
    security(("api_key" = []))
)]
async fn list_vectors(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<VectorListEntry>>, (StatusCode, Json<ErrorResponse>)> {
    let collection = state.db.get_collection(&name).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(10).min(100);

    let result = tokio::task::spawn_blocking(move || collection.list(offset, limit))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(
        result
            .into_iter()
            .map(|(id, metadata)| VectorListEntry {
                id: id.to_string(),
                metadata,
            })
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/collections/{name}/search",
    params(
        ("name" = String, Path, description = "Collection name")
    ),
    request_body = SearchRequest,
    responses(
        (status = 200, description = "List of nearest neighbors", body = [SearchResult]),
        (status = 400, description = "Invalid request", body = ErrorResponse)
    ),
    security(("api_key" = []))
)]
async fn search_vector(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<SearchRequest>,
) -> Result<Json<Vec<SearchResult>>, (StatusCode, Json<ErrorResponse>)> {
    let collection = state.db.get_collection(&name).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let result = tokio::task::spawn_blocking(move || {
        collection.search(&payload.vector, payload.k, payload.filter.as_ref())
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    match result {
        Ok(results) => {
            let response = results
                .into_iter()
                .map(|(id, distance, metadata)| SearchResult {
                    id: id.as_str().to_string(),
                    distance,
                    metadata,
                })
                .collect();
            Ok(Json(response))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}
