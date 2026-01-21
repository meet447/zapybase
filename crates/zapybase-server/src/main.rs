use axum::{
    extract::{State, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use parking_lot::RwLock;
use zapybase_core::{Config, VectorDb};

#[derive(Clone)]
struct AppState {
    db: Arc<RwLock<VectorDb>>,
}

#[derive(Deserialize)]
struct InsertRequest {
    id: String,
    vector: Vec<f32>,
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
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::default();
    let db = VectorDb::new(config).expect("Failed to initialize database");
    let state = AppState {
        db: Arc::new(RwLock::new(db)),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/vectors", post(insert_vector))
        .route("/search", post(search_vector))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn insert_vector(
    State(state): State<AppState>,
    Json(payload): Json<InsertRequest>,
) -> Result<&'static str, String> {
    let result = tokio::task::spawn_blocking(move || {
        let mut db = state.db.write();
        db.insert(payload.id, &payload.vector)
    }).await.map_err(|e| e.to_string())?;

    match result {
        Ok(_) => Ok("Inserted"),
        Err(e) => Err(format!("Insert failed: {:?}", e)),
    }
}

async fn search_vector(
    State(state): State<AppState>,
    Json(payload): Json<SearchRequest>,
) -> Result<Json<Vec<SearchResult>>, String> {
    let result = tokio::task::spawn_blocking(move || {
        let db = state.db.read();
        db.search(&payload.vector, payload.k)
    }).await.map_err(|e| e.to_string())?;

    match result {
        Ok(results) => {
            let response = results
                .into_iter()
                .map(|(id, distance)| SearchResult {
                    id: id.as_str().to_string(),
                    distance,
                })
                .collect();
            Ok(Json(response))
        }
        Err(e) => Err(format!("Search failed: {:?}", e)),
    }
}
