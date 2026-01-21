use axum::{
    extract::{State, Json, Path},
    routing::{get, post},
    Router,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use zapybase_core::{Config, Database, DistanceMetric};

#[derive(Clone)]
struct AppState {
    db: Arc<Database>,
}

#[derive(Deserialize)]
struct CreateCollectionRequest {
    name: String,
    dimensions: usize,
    #[serde(default)]
    distance_metric: DistanceMetric,
}

#[derive(Deserialize)]
struct InsertRequest {
    id: String,
    vector: Vec<f32>,
    metadata: Option<Value>,
}

#[derive(Deserialize)]
struct SearchRequest {
    vector: Vec<f32>,
    k: usize,
}

#[derive(Serialize)]
struct SearchResult {
    id: String,
    distance: f32,
    metadata: Option<Value>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db = Database::new();
    let state = AppState {
        db: Arc::new(db),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/collections", post(create_collection).get(list_collections))
        .route("/collections/:name/vectors", post(insert_vector))
        .route("/collections/:name/search", post(search_vector))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn create_collection(
    State(state): State<AppState>,
    Json(payload): Json<CreateCollectionRequest>,
) -> Result<&'static str, (StatusCode, Json<ErrorResponse>)> {
    let config = Config {
        dimensions: payload.dimensions,
        distance_metric: payload.distance_metric,
        ..Config::default()
    };

    match state.db.create_collection(&payload.name, config) {
        Ok(_) => Ok("Created"),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

async fn list_collections(State(state): State<AppState>) -> Json<Vec<String>> {
    Json(state.db.list_collections())
}

async fn insert_vector(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<InsertRequest>,
) -> Result<&'static str, (StatusCode, Json<ErrorResponse>)> {
    let collection = state.db.get_collection(&name).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: e.to_string() }),
        )
    })?;

    let result = tokio::task::spawn_blocking(move || {
        let mut db = collection.write();
        db.insert(payload.id, &payload.vector, payload.metadata)
    }).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: e.to_string() }),
    ))?;

    match result {
        Ok(_) => Ok("Inserted"),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

async fn search_vector(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<SearchRequest>,
) -> Result<Json<Vec<SearchResult>>, (StatusCode, Json<ErrorResponse>)> {
    let collection = state.db.get_collection(&name).map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: e.to_string() }),
        )
    })?;

    let result = tokio::task::spawn_blocking(move || {
        let db = collection.read();
        db.search(&payload.vector, payload.k)
    }).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: e.to_string() }),
    ))?;

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
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}
