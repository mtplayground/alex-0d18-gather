use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    database: &'static str,
}

pub fn app(state: AppState) -> Router {
    let protected_auth_routes = crate::auth::routes::protected_router(state.clone());
    let protected_event_routes = crate::events::protected_router(state.clone());

    Router::new()
        .route("/", get(root))
        .route("/api/health", get(health))
        .nest("/api/auth", crate::auth::routes::router())
        .nest("/api/auth", protected_auth_routes)
        .nest("/api/events", protected_event_routes)
        .with_state(state)
        .layer(CorsLayer::new().allow_origin(Any))
}

async fn root() -> &'static str {
    "Gather API"
}

async fn health(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    match sqlx::query_scalar::<_, bool>("SELECT TRUE")
        .fetch_one(&state.db)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(HealthResponse {
                status: "ok",
                service: "gather-api",
                database: "ok",
            }),
        ),
        Err(error) => {
            tracing::error!(%error, "database health check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "error",
                    service: "gather-api",
                    database: "unavailable",
                }),
            )
        }
    }
}
