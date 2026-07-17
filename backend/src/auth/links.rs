use crate::config::{AuthConfig, ServerConfig};

const DEFAULT_PUBLIC_BASE_URL: &str = "http://localhost:8080";

#[derive(Debug, Clone)]
pub struct AuthLinkConfig {
    auth_url: String,
    app_token: String,
    public_base_url: String,
}

impl AuthLinkConfig {
    pub fn from_config(server: &ServerConfig, auth: &AuthConfig) -> Self {
        Self {
            auth_url: auth.url.trim_end_matches('/').to_owned(),
            app_token: auth.app_token.clone(),
            public_base_url: server
                .self_url
                .as_deref()
                .unwrap_or(DEFAULT_PUBLIC_BASE_URL)
                .trim_end_matches('/')
                .to_owned(),
        }
    }

    pub fn registration_url(&self, return_path: Option<&str>) -> anyhow::Result<String> {
        self.auth_url_for_return_path(return_path)
    }

    pub fn login_url(&self, return_path: Option<&str>) -> anyhow::Result<String> {
        self.auth_url_for_return_path(return_path)
    }

    pub fn google_oauth_url(&self, return_path: Option<&str>) -> anyhow::Result<String> {
        self.auth_url_for_return_path(return_path)
    }

    fn auth_url_for_return_path(&self, return_path: Option<&str>) -> anyhow::Result<String> {
        let path = normalize_return_path(return_path)?;
        let return_to = format!("{}{}", self.public_base_url, path);

        Ok(format!(
            "{}/login?app_token={}&return_to={}",
            self.auth_url,
            urlencoding::encode(&self.app_token),
            urlencoding::encode(&return_to)
        ))
    }
}

fn normalize_return_path(return_path: Option<&str>) -> anyhow::Result<&str> {
    let Some(path) = return_path.map(str::trim).filter(|path| !path.is_empty()) else {
        return Ok("/");
    };

    if !path.starts_with('/') || path.starts_with("//") {
        return Err(anyhow::anyhow!(
            "return_to must be an absolute same-origin frontend path"
        ));
    }
    if path == "/api" || path.starts_with("/api/") {
        return Err(anyhow::anyhow!(
            "return_to must point to a user-visible frontend page"
        ));
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::AuthLinkConfig;
    use crate::config::{AuthConfig, ServerConfig};

    fn config() -> AuthLinkConfig {
        AuthLinkConfig::from_config(
            &ServerConfig {
                host: "0.0.0.0".to_owned(),
                port: 8080,
                self_url: Some("https://gather.example".to_owned()),
                allowed_cors_origin: None,
            },
            &AuthConfig {
                url: "https://auth.mctai.app/".to_owned(),
                app_token: "app_test_token".to_owned(),
                jwks_url: "https://auth.mctai.app/.well-known/jwks.json".to_owned(),
            },
        )
    }

    #[test]
    fn builds_registration_link_to_frontend_page() {
        let url = config()
            .registration_url(Some("/dashboard"))
            .expect("registration link should build");

        assert_eq!(
            url,
            "https://auth.mctai.app/login?app_token=app_test_token&return_to=https%3A%2F%2Fgather.example%2Fdashboard"
        );
    }

    #[test]
    fn defaults_to_root_page() {
        let url = config()
            .registration_url(None)
            .expect("registration link should build");

        assert!(url.ends_with("return_to=https%3A%2F%2Fgather.example%2F"));
    }

    #[test]
    fn builds_login_link_to_frontend_page() {
        let url = config()
            .login_url(Some("/profile"))
            .expect("login link should build");

        assert_eq!(
            url,
            "https://auth.mctai.app/login?app_token=app_test_token&return_to=https%3A%2F%2Fgather.example%2Fprofile"
        );
    }

    #[test]
    fn builds_google_oauth_link_to_frontend_page() {
        let url = config()
            .google_oauth_url(Some("/dashboard"))
            .expect("google oauth link should build");

        assert_eq!(
            url,
            "https://auth.mctai.app/login?app_token=app_test_token&return_to=https%3A%2F%2Fgather.example%2Fdashboard"
        );
    }

    #[test]
    fn rejects_api_return_targets() {
        assert!(config().registration_url(Some("/api/me")).is_err());
    }
}
