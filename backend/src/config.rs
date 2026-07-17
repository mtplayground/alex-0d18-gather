use sqlx::postgres::PgSslMode;
use std::{env, fmt, net::SocketAddr, time::Duration};

#[derive(Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub object_storage: ObjectStorageConfig,
    pub auth: AuthConfig,
    pub email: Option<EmailConfig>,
    pub legacy_jwt_secret: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub self_url: Option<String>,
    pub allowed_cors_origin: Option<String>,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub ssl_mode: Option<PgSslMode>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct ObjectStorageConfig {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket: String,
    pub prefix: String,
    pub endpoint: String,
    pub region: String,
    pub force_path_style: bool,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct AuthConfig {
    pub url: String,
    pub app_token: String,
    pub jwks_url: String,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct EmailConfig {
    pub url: String,
    pub app_token: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            server: ServerConfig::from_env()?,
            database: DatabaseConfig::from_env()?,
            object_storage: ObjectStorageConfig::from_env()?,
            auth: AuthConfig::from_env()?,
            email: EmailConfig::from_env()?,
            legacy_jwt_secret: optional_env("JWT_SECRET"),
        })
    }
}

impl ServerConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
        let port = match env::var("PORT") {
            Ok(value) => value
                .parse::<u16>()
                .map_err(|error| anyhow::anyhow!("PORT must be a valid u16: {error}"))?,
            Err(_) => 8080,
        };

        Ok(Self {
            host,
            port,
            self_url: optional_env("SELF_URL"),
            allowed_cors_origin: optional_env("ALLOWED_CORS_ORIGIN"),
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
        let url = required_env("DATABASE_URL")?;
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

impl ObjectStorageConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let prefix = required_env("OBJECT_STORAGE_PREFIX")?;
        if prefix.is_empty() {
            return Err(anyhow::anyhow!("OBJECT_STORAGE_PREFIX must not be empty"));
        }

        Ok(Self {
            access_key_id: required_env("OBJECT_STORAGE_ACCESS_KEY_ID")?,
            secret_access_key: required_env("OBJECT_STORAGE_SECRET_ACCESS_KEY")?,
            bucket: required_env("OBJECT_STORAGE_BUCKET")?,
            prefix,
            endpoint: required_env("OBJECT_STORAGE_ENDPOINT")?,
            region: required_env("OBJECT_STORAGE_REGION")?,
            force_path_style: parse_required_env("OBJECT_STORAGE_FORCE_PATH_STYLE")?,
        })
    }
}

impl AuthConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            url: required_env("MCTAI_AUTH_URL")?,
            app_token: required_env("MCTAI_AUTH_APP_TOKEN")?,
            jwks_url: required_env("MCTAI_AUTH_JWKS_URL")?,
        })
    }
}

impl EmailConfig {
    pub fn from_env() -> anyhow::Result<Option<Self>> {
        match (
            optional_env("MCTAI_EMAIL_URL"),
            optional_env("MCTAI_EMAIL_APP_TOKEN"),
        ) {
            (Some(url), Some(app_token)) => Ok(Some(Self { url, app_token })),
            (None, None) => Ok(None),
            _ => Err(anyhow::anyhow!(
                "MCTAI_EMAIL_URL and MCTAI_EMAIL_APP_TOKEN must be set together"
            )),
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Config")
            .field("server", &self.server)
            .field("database", &self.database)
            .field("object_storage", &self.object_storage)
            .field("auth", &self.auth)
            .field("email", &self.email)
            .field(
                "legacy_jwt_secret",
                &redacted_option(&self.legacy_jwt_secret),
            )
            .finish()
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

impl fmt::Debug for ObjectStorageConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ObjectStorageConfig")
            .field("access_key_id", &"<redacted>")
            .field("secret_access_key", &"<redacted>")
            .field("bucket", &self.bucket)
            .field("prefix", &self.prefix)
            .field("endpoint", &self.endpoint)
            .field("region", &self.region)
            .field("force_path_style", &self.force_path_style)
            .finish()
    }
}

impl fmt::Debug for AuthConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthConfig")
            .field("url", &self.url)
            .field("app_token", &"<redacted>")
            .field("jwks_url", &self.jwks_url)
            .finish()
    }
}

impl fmt::Debug for EmailConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EmailConfig")
            .field("url", &self.url)
            .field("app_token", &"<redacted>")
            .finish()
    }
}

fn required_env(key: &str) -> anyhow::Result<String> {
    env::var(key).map_err(|_| anyhow::anyhow!("{key} env var is required"))
}

fn optional_env(key: &str) -> Option<String> {
    env::var(key).ok().filter(|value| !value.is_empty())
}

fn parse_required_env<T>(key: &str) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    required_env(key)?
        .parse::<T>()
        .map_err(|error| anyhow::anyhow!("{key} must be valid: {error}"))
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

fn redacted_option(value: &Option<String>) -> &'static str {
    if value.is_some() {
        "<redacted>"
    } else {
        "<unset>"
    }
}
