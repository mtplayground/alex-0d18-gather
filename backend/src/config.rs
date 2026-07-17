use sqlx::postgres::PgSslMode;
use std::{env, fmt, net::SocketAddr, time::Duration};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database: DatabaseConfig,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub ssl_mode: Option<PgSslMode>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
        let port = match env::var("PORT") {
            Ok(value) => value
                .parse::<u16>()
                .map_err(|error| anyhow::anyhow!("PORT must be a valid u16: {error}"))?,
            Err(_) => 8080,
        };
        let database = DatabaseConfig::from_env()?;

        Ok(Self {
            host,
            port,
            database,
        })
    }

    pub fn socket_addr(&self) -> anyhow::Result<SocketAddr> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|error| anyhow::anyhow!("invalid bind address: {error}"))
    }
}

impl DatabaseConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let url = env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL env var is required"))?;
        let max_connections = parse_optional_env("DATABASE_MAX_CONNECTIONS", 5)?;
        let acquire_timeout_seconds = parse_optional_env("DATABASE_ACQUIRE_TIMEOUT_SECONDS", 10)?;
        let ssl_mode = match env::var("DATABASE_SSL_MODE") {
            Ok(value) => Some(parse_database_ssl_mode(&value)?),
            Err(_) => None,
        };

        Ok(Self {
            url,
            max_connections,
            acquire_timeout: Duration::from_secs(acquire_timeout_seconds),
            ssl_mode,
        })
    }
}

impl fmt::Debug for DatabaseConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DatabaseConfig")
            .field("url", &"<redacted>")
            .field("max_connections", &self.max_connections)
            .field("acquire_timeout", &self.acquire_timeout)
            .field("ssl_mode", &self.ssl_mode)
            .finish()
    }
}

fn parse_optional_env<T>(key: &str, default: T) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(value) => value
            .parse::<T>()
            .map_err(|error| anyhow::anyhow!("{key} must be valid: {error}")),
        Err(_) => Ok(default),
    }
}

fn parse_database_ssl_mode(value: &str) -> anyhow::Result<PgSslMode> {
    match value {
        "disable" => Ok(PgSslMode::Disable),
        "prefer" => Ok(PgSslMode::Prefer),
        "require" => Ok(PgSslMode::Require),
        "verify-ca" => Ok(PgSslMode::VerifyCa),
        "verify-full" => Ok(PgSslMode::VerifyFull),
        _ => Err(anyhow::anyhow!(
            "DATABASE_SSL_MODE must be one of disable, prefer, require, verify-ca, verify-full"
        )),
    }
}
