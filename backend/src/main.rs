mod auth;
mod config;
mod db;
mod email;
mod events;
mod models;
mod routes;
mod state;
mod storage;

use anyhow::Context;
use config::Config;
use state::AppState;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env().context("failed to load server configuration")?;
    let pool = db::connect(&config.database)
        .await
        .context("failed to connect to PostgreSQL")?;
    db::run_migrations(&pool)
        .await
        .context("failed to run database migrations")?;
    let storage = storage::ObjectStorage::from_config(&config.object_storage)
        .context("failed to configure object storage client")?;
    let email = email::EmailClient::from_config(config.email.as_ref());
    let auth_links = auth::links::AuthLinkConfig::from_config(&config.server, &config.auth);
    let auth = auth::middleware::AuthVerifier::from_config(&config.auth);

    let addr = config.server.socket_addr()?;
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind API server on {addr}"))?;
    let state = AppState::new(pool, storage, email, auth_links, auth);

    tracing::info!(%addr, "Gather API listening");

    axum::serve(
        listener,
        routes::app(state)
            .layer(TraceLayer::new_for_http())
            .into_make_service(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("API server exited with an error")
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::warn!(%error, "failed to install Ctrl+C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => tracing::warn!(%error, "failed to install SIGTERM handler"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
