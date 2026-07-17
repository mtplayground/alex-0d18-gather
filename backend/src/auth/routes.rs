use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    email::{EmailMessage, EmailSendOutcome},
    state::AppState,
};

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    email: String,
    return_to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: Option<String>,
    return_to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OAuthStartQuery {
    return_to: Option<String>,
}

#[derive(Debug, Serialize)]
struct RegisterResponse {
    status: &'static str,
    auth_url: String,
    email_sent: bool,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    status: &'static str,
    auth_url: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: &'static str,
    message: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/google", get(google_oauth))
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), RegisterError> {
    let email = normalize_email(&payload.email)?;
    let auth_url = state
        .auth_links
        .registration_url(payload.return_to.as_deref())
        .map_err(RegisterError::BadRequest)?;

    let message = verification_message(&email, &auth_url);
    let email_sent = match state.email.send(message).await {
        Ok(EmailSendOutcome::Sent { message_id }) => {
            tracing::info!(%message_id, email = %email, "registration verification email sent");
            true
        }
        Ok(EmailSendOutcome::Skipped { reason }) => {
            tracing::warn!(%reason, email = %email, "registration verification email skipped");
            false
        }
        Err(error) => return Err(RegisterError::EmailSend(error)),
    };

    Ok((
        StatusCode::ACCEPTED,
        Json(RegisterResponse {
            status: if email_sent {
                "verification_email_sent"
            } else {
                "registration_started"
            },
            auth_url,
            email_sent,
        }),
    ))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), RegisterError> {
    if let Some(email) = payload.email.as_deref() {
        let email = normalize_email(email)?;
        tracing::info!(email = %email, "login started through myClawTeam auth");
    }

    let auth_url = state
        .auth_links
        .login_url(payload.return_to.as_deref())
        .map_err(RegisterError::BadRequest)?;

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            status: "login_started",
            auth_url,
        }),
    ))
}

async fn google_oauth(
    State(state): State<AppState>,
    Query(query): Query<OAuthStartQuery>,
) -> Result<Redirect, RegisterError> {
    let auth_url = state
        .auth_links
        .google_oauth_url(query.return_to.as_deref())
        .map_err(RegisterError::BadRequest)?;

    tracing::info!("google sign-in delegated to myClawTeam auth");
    Ok(Redirect::temporary(&auth_url))
}

fn normalize_email(email: &str) -> Result<String, RegisterError> {
    let normalized = email.trim().to_lowercase();
    if normalized.is_empty()
        || normalized.len() > 320
        || normalized.chars().any(char::is_whitespace)
        || !normalized.contains('@')
    {
        return Err(RegisterError::BadRequest(anyhow::anyhow!(
            "email must be a valid address"
        )));
    }

    let Some((local, domain)) = normalized.split_once('@') else {
        return Err(RegisterError::BadRequest(anyhow::anyhow!(
            "email must be a valid address"
        )));
    };
    if local.is_empty() || domain.is_empty() || !domain.contains('.') {
        return Err(RegisterError::BadRequest(anyhow::anyhow!(
            "email must be a valid address"
        )));
    }

    Ok(normalized)
}

fn verification_message(email: &str, auth_url: &str) -> EmailMessage {
    let html = format!(
        r#"
        <p>Welcome to Gather.</p>
        <p>Use this secure link to verify your email address and finish registration:</p>
        <p><a href="{auth_url}">Complete registration</a></p>
        <p>If you did not request this, you can ignore this email.</p>
        "#
    );
    let text = format!(
        "Welcome to Gather.\n\nUse this secure link to verify your email address and finish registration:\n{auth_url}\n\nIf you did not request this, you can ignore this email."
    );

    EmailMessage {
        to: vec![email.to_owned()],
        subject: "Complete your Gather registration".to_owned(),
        html: Some(html),
        text: Some(text),
        reply_to: None,
    }
}

#[derive(Debug)]
enum RegisterError {
    BadRequest(anyhow::Error),
    EmailSend(anyhow::Error),
}

impl IntoResponse for RegisterError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::BadRequest(error) => (StatusCode::BAD_REQUEST, "bad_request", error.to_string()),
            Self::EmailSend(error) => {
                tracing::error!(%error, "failed to send registration verification email");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "email_unavailable",
                    "verification email could not be sent; try again shortly".to_owned(),
                )
            }
        };

        (status, Json(ErrorResponse { code, message })).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_email;

    #[test]
    fn normalizes_valid_email() {
        assert_eq!(
            normalize_email(" PERSON@Example.COM ").expect("email should normalize"),
            "person@example.com"
        );
    }

    #[test]
    fn rejects_invalid_email() {
        assert!(normalize_email("missing-at.example").is_err());
        assert!(normalize_email("person@example").is_err());
        assert!(normalize_email("person @example.com").is_err());
    }
}
