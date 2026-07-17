use axum::{
    extract::State,
    http::{HeaderValue, StatusCode},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    services::{ServeDir, ServeFile},
};

use crate::{config::ServerConfig, state::AppState};

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    database: &'static str,
}

pub fn app(state: AppState, server: &ServerConfig) -> anyhow::Result<Router> {
    let protected_auth_routes = crate::auth::routes::protected_router(state.clone());
    let protected_event_routes = crate::events::protected_router(state.clone());
    let protected_invitation_routes = crate::events::invitation_router(state.clone());

    let router = Router::new()
        .route("/api/health", get(health))
        .nest("/api/auth", crate::auth::routes::router())
        .nest("/api/auth", protected_auth_routes)
        .nest("/api/events", protected_event_routes)
        .nest("/api/invitations", protected_invitation_routes);

    let router = if let Some(frontend_dir) = server.frontend_dist_dir.as_ref() {
        let index_path = frontend_dir.join("index.html");
        if !index_path.is_file() {
            return Err(anyhow::anyhow!(
                "FRONTEND_DIST_DIR must contain index.html: {}",
                index_path.display()
            ));
        }
        tracing::info!(frontend_dist_dir = %frontend_dir.display(), "serving frontend assets");
        router.fallback_service(ServeDir::new(frontend_dir).fallback(ServeFile::new(index_path)))
    } else {
        router.route("/", get(root))
    };

    Ok(router.with_state(state).layer(cors_layer(server)?))
}

async fn root() -> &'static str {
    "Gather API"
}

fn cors_layer(server: &ServerConfig) -> anyhow::Result<CorsLayer> {
    let allowed_origin = match server.allowed_cors_origin.as_deref() {
        Some(origin) => AllowOrigin::exact(
            HeaderValue::from_str(origin)
                .map_err(|error| anyhow::anyhow!("ALLOWED_CORS_ORIGIN is invalid: {error}"))?,
        ),
        None => AllowOrigin::from(Any),
    };

    Ok(CorsLayer::new().allow_origin(allowed_origin))
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
