use std::{env, net::SocketAddr};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
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

        Ok(Self { host, port })
    }

    pub fn socket_addr(&self) -> anyhow::Result<SocketAddr> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|error| anyhow::anyhow!("invalid bind address: {error}"))
    }
}

