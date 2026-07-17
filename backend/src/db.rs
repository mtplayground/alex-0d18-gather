use std::str::FromStr;

use sqlx::{
    migrate::Migrator,
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};

use crate::config::DatabaseConfig;

static MIGRATOR: Migrator = sqlx::migrate!("../migrations");

pub async fn connect(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    let mut options = PgConnectOptions::from_str(&config.url)?;
    if let Some(ssl_mode) = config.ssl_mode {
        options = options.ssl_mode(ssl_mode);
    }

    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.acquire_timeout)
        .connect_with(options)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}
