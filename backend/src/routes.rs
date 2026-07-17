use axum::{routing::get, Json, Router};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

pub fn app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/api/health", get(health))
        .layer(CorsLayer::new().allow_origin(Any))
}

async fn root() -> &'static str {
    "Gather API"
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "gather-api",
    })
}

