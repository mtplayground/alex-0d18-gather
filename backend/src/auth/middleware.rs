use std::{sync::Arc, time::Duration};

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::time::Instant;

use crate::{
    auth::session::{upsert_user_from_mctai_claims, MctaiSessionClaims},
    config::AuthConfig,
    state::AppState,
};

const SESSION_COOKIE_NAME: &str = "mctai_session";
const JWKS_CACHE_TTL: Duration = Duration::from_secs(300);

#[derive(Clone)]
pub struct AuthVerifier {
    inner: Arc<AuthVerifierInner>,
}

struct AuthVerifierInner {
    http: reqwest::Client,
    jwks_url: String,
    issuer: String,
    audience: String,
    jwks_cache: RwLock<JwksCache>,
}

#[derive(Debug, Default)]
struct JwksCache {
    fetched_at: Option<Instant>,
    keys: Vec<JwkKey>,
}

#[derive(Debug, Clone, Deserialize)]
struct JwkSet {
    keys: Vec<JwkKey>,
}

#[derive(Debug, Clone, Deserialize)]
struct JwkKey {
    kid: String,
    kty: String,
    n: String,
    e: String,
}

#[derive(Debug, Clone, Deserialize)]
struct MctaiJwtClaims {
    sub: String,
    email: String,
    #[serde(default)]
    email_verified: bool,
    name: Option<String>,
    picture: Option<String>,
}

#[derive(Debug, Serialize)]
struct AuthErrorResponse {
    code: &'static str,
    message: &'static str,
}

#[derive(Debug)]
pub enum AuthError {
    MissingSession,
    InvalidSession(anyhow::Error),
    UserUpsert(sqlx::Error),
}

impl AuthVerifier {
    pub fn from_config(config: &AuthConfig) -> Self {
        Self {
            inner: Arc::new(AuthVerifierInner {
                http: reqwest::Client::new(),
                jwks_url: config.jwks_url.clone(),
                issuer: config.url.clone(),
                audience: config.app_token.clone(),
                jwks_cache: RwLock::new(JwksCache::default()),
            }),
        }
    }

    pub async fn verify_token(&self, token: &str) -> Result<MctaiSessionClaims, AuthError> {
        let header = decode_header(token)
            .map_err(|error| AuthError::InvalidSession(anyhow::anyhow!(error)))?;
        let kid = header.kid.ok_or_else(|| {
            AuthError::InvalidSession(anyhow::anyhow!("session token missing key id"))
        })?;
        let key = self.decoding_key(&kid).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.inner.audience.as_str()]);
        validation.set_issuer(&[self.inner.issuer.as_str()]);

        let token_data = decode::<MctaiJwtClaims>(token, &key, &validation)
            .map_err(|error| AuthError::InvalidSession(anyhow::anyhow!(error)))?;

        Ok(MctaiSessionClaims {
            sub: token_data.claims.sub,
            email: token_data.claims.email,
            email_verified: token_data.claims.email_verified,
            name: token_data.claims.name,
            picture: token_data.claims.picture,
        })
    }

    async fn decoding_key(&self, kid: &str) -> Result<DecodingKey, AuthError> {
        let keys = self.jwks_keys().await?;
        let key = keys
            .iter()
            .find(|key| key.kid == kid && key.kty == "RSA")
            .ok_or_else(|| AuthError::InvalidSession(anyhow::anyhow!("session key not found")))?;

        DecodingKey::from_rsa_components(&key.n, &key.e)
            .map_err(|error| AuthError::InvalidSession(anyhow::anyhow!(error)))
    }

    async fn jwks_keys(&self) -> Result<Vec<JwkKey>, AuthError> {
        {
            let cache = self.inner.jwks_cache.read().await;
            if cache
                .fetched_at
                .is_some_and(|fetched_at| fetched_at.elapsed() < JWKS_CACHE_TTL)
                && !cache.keys.is_empty()
            {
                return Ok(cache.keys.clone());
            }
        }

        let response = self
            .inner
            .http
            .get(&self.inner.jwks_url)
            .send()
            .await
            .map_err(|error| AuthError::InvalidSession(anyhow::anyhow!(error)))?;

        if !response.status().is_success() {
            return Err(AuthError::InvalidSession(anyhow::anyhow!(
                "JWKS endpoint returned {}",
                response.status()
            )));
        }

        let jwks = response
            .json::<JwkSet>()
            .await
            .map_err(|error| AuthError::InvalidSession(anyhow::anyhow!(error)))?;
        let mut cache = self.inner.jwks_cache.write().await;
        cache.fetched_at = Some(Instant::now());
        cache.keys = jwks.keys;

        Ok(cache.keys.clone())
    }
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = extract_session_cookie(request.headers()).ok_or(AuthError::MissingSession)?;
    let claims = state.auth.verify_token(token).await?;
    let session = upsert_user_from_mctai_claims(&state.db, &claims)
        .await
        .map_err(AuthError::UserUpsert)?;

    request.extensions_mut().insert(session);
    Ok(next.run(request).await)
}

pub fn extract_session_cookie(headers: &axum::http::HeaderMap) -> Option<&str> {
    let cookies = headers.get(header::COOKIE)?.to_str().ok()?;

    cookies.split(';').find_map(|cookie| {
        let (name, value) = cookie.trim().split_once('=')?;
        (name == SESSION_COOKIE_NAME && !value.is_empty()).then_some(value)
    })
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            Self::MissingSession => (
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse {
                    code: "unauthorized",
                    message: "authentication required",
                }),
            )
                .into_response(),
            Self::InvalidSession(error) => {
                tracing::warn!(%error, "session validation failed");
                (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthErrorResponse {
                        code: "invalid_session",
                        message: "session is invalid or expired",
                    }),
                )
                    .into_response()
            }
            Self::UserUpsert(error) => {
                tracing::error!(%error, "failed to upsert authenticated user");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AuthErrorResponse {
                        code: "auth_user_sync_failed",
                        message: "authenticated user could not be synchronized",
                    }),
                )
                    .into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::extract_session_cookie;

    #[test]
    fn extracts_mctai_session_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            HeaderValue::from_static("theme=dark; mctai_session=abc.def.ghi; other=value"),
        );

        assert_eq!(extract_session_cookie(&headers), Some("abc.def.ghi"));
    }

    #[test]
    fn ignores_missing_or_empty_session_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            HeaderValue::from_static("mctai_session=; other=value"),
        );

        assert_eq!(extract_session_cookie(&headers), None);
    }
}
