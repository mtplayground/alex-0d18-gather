use axum::{
    extract::{DefaultBodyLimit, Extension, Multipart, Query, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::{
    auth::{middleware::require_auth, session::AuthenticatedSession},
    email::{templates, EmailSendOutcome},
    models::user::UserProfile,
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

#[derive(Debug, Deserialize)]
struct PasswordResetRequest {
    email: String,
    return_to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PasswordResetConfirmRequest {
    return_to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateProfileRequest {
    #[serde(default)]
    display_name: Option<Option<String>>,
    #[serde(default)]
    full_name: Option<Option<String>>,
    #[serde(default)]
    bio: Option<Option<String>>,
    #[serde(default)]
    location: Option<Option<String>>,
    #[serde(default)]
    website_url: Option<Option<String>>,
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
struct PasswordResetRequestResponse {
    status: &'static str,
    auth_url: String,
    email_sent: bool,
}

#[derive(Debug, Serialize)]
struct PasswordResetConfirmResponse {
    status: &'static str,
    auth_url: String,
}

#[derive(Debug, Serialize)]
struct ProfileResponse {
    #[serde(flatten)]
    profile: UserProfile,
    avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct PhotoUploadResponse {
    status: &'static str,
    profile: ProfileResponse,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: &'static str,
    message: String,
}

const MAX_PROFILE_PHOTO_BYTES: usize = 5 * 1024 * 1024;
const AVATAR_SIGNED_URL_TTL: Duration = Duration::from_secs(60 * 60);

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/google", get(google_oauth))
        .route("/password-reset/request", post(request_password_reset))
        .route("/password-reset/confirm", post(confirm_password_reset))
}

pub fn protected_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me", get(current_user))
        .route("/profile", get(current_user).patch(update_profile))
        .route("/profile/photo", post(upload_profile_photo))
        .layer(DefaultBodyLimit::max(MAX_PROFILE_PHOTO_BYTES + 64 * 1024))
        .route_layer(middleware::from_fn_with_state(state, require_auth))
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

    let message = templates::verification(&email, &auth_url);
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

async fn request_password_reset(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetRequest>,
) -> Result<(StatusCode, Json<PasswordResetRequestResponse>), RegisterError> {
    let email = normalize_email(&payload.email)?;
    let auth_url = state
        .auth_links
        .password_reset_url(payload.return_to.as_deref())
        .map_err(RegisterError::BadRequest)?;

    let message = templates::password_reset(&email, &auth_url);
    let email_sent = match state.email.send(message).await {
        Ok(EmailSendOutcome::Sent { message_id }) => {
            tracing::info!(%message_id, email = %email, "password reset email sent");
            true
        }
        Ok(EmailSendOutcome::Skipped { reason }) => {
            tracing::warn!(%reason, email = %email, "password reset email skipped");
            false
        }
        Err(error) => return Err(RegisterError::EmailSend(error)),
    };

    Ok((
        StatusCode::ACCEPTED,
        Json(PasswordResetRequestResponse {
            status: if email_sent {
                "password_reset_email_sent"
            } else {
                "password_reset_started"
            },
            auth_url,
            email_sent,
        }),
    ))
}

async fn confirm_password_reset(
    State(state): State<AppState>,
    Json(payload): Json<PasswordResetConfirmRequest>,
) -> Result<(StatusCode, Json<PasswordResetConfirmResponse>), RegisterError> {
    let auth_url = state
        .auth_links
        .password_reset_url(payload.return_to.as_deref())
        .map_err(RegisterError::BadRequest)?;

    Ok((
        StatusCode::OK,
        Json(PasswordResetConfirmResponse {
            status: "password_reset_managed_by_auth_service",
            auth_url,
        }),
    ))
}

async fn current_user(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
) -> Result<Json<ProfileResponse>, ProfileError> {
    Ok(Json(profile_response(&state, session.user).await?))
}

async fn update_profile(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, ProfileError> {
    let profile = normalize_profile_update(payload)?;
    let user = sqlx::query_as::<_, crate::models::user::User>(
        r#"
        UPDATE users
        SET
            display_name = CASE WHEN $2 THEN $3 ELSE display_name END,
            full_name = CASE WHEN $4 THEN $5 ELSE full_name END,
            bio = CASE WHEN $6 THEN $7 ELSE bio END,
            location = CASE WHEN $8 THEN $9 ELSE location END,
            website_url = CASE WHEN $10 THEN $11 ELSE website_url END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            email,
            password_hash,
            oauth_provider,
            oauth_subject,
            display_name,
            full_name,
            bio,
            location,
            website_url,
            avatar_object_key,
            email_verified,
            email_verified_at,
            created_at,
            updated_at,
            last_seen_at
        "#,
    )
    .bind(session.user.id)
    .bind(profile.display_name.provided)
    .bind(profile.display_name.value)
    .bind(profile.full_name.provided)
    .bind(profile.full_name.value)
    .bind(profile.bio.provided)
    .bind(profile.bio.value)
    .bind(profile.location.provided)
    .bind(profile.location.value)
    .bind(profile.website_url.provided)
    .bind(profile.website_url.value)
    .fetch_one(&state.db)
    .await
    .map_err(ProfileError::Database)?;

    Ok(Json(profile_response(&state, user).await?))
}

async fn upload_profile_photo(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<PhotoUploadResponse>), ProfileError> {
    let photo = extract_profile_photo(&mut multipart).await?;
    let extension = image_extension(&photo.content_type)?;
    let relative_key = format!(
        "avatars/{}/{}.{}",
        session.user.id,
        Uuid::new_v4(),
        extension
    );
    let stored = state
        .storage
        .put_object(
            &relative_key,
            photo.bytes,
            Some(photo.content_type.as_str()),
        )
        .await
        .map_err(ProfileError::Storage)?;

    let user = sqlx::query_as::<_, crate::models::user::User>(
        r#"
        UPDATE users
        SET
            avatar_object_key = $2,
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            email,
            password_hash,
            oauth_provider,
            oauth_subject,
            display_name,
            full_name,
            bio,
            location,
            website_url,
            avatar_object_key,
            email_verified,
            email_verified_at,
            created_at,
            updated_at,
            last_seen_at
        "#,
    )
    .bind(session.user.id)
    .bind(&stored.relative_key)
    .fetch_one(&state.db)
    .await
    .map_err(ProfileError::Database)?;

    if let Some(previous_key) = session.user.avatar_object_key.as_deref() {
        if previous_key != stored.relative_key {
            if let Err(error) = state.storage.delete_object(previous_key).await {
                tracing::warn!(%error, user_id = %session.user.id, "failed to delete previous avatar object");
            }
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(PhotoUploadResponse {
            status: "profile_photo_uploaded",
            profile: profile_response(&state, user).await?,
        }),
    ))
}

struct NormalizedProfileUpdate {
    display_name: ProfilePatchValue,
    full_name: ProfilePatchValue,
    bio: ProfilePatchValue,
    location: ProfilePatchValue,
    website_url: ProfilePatchValue,
}

#[derive(Debug, PartialEq, Eq)]
struct ProfilePatchValue {
    provided: bool,
    value: Option<String>,
}

struct ProfilePhotoUpload {
    bytes: Bytes,
    content_type: String,
}

async fn profile_response(
    state: &AppState,
    user: crate::models::user::User,
) -> Result<ProfileResponse, ProfileError> {
    let avatar_url = match user.avatar_object_key.as_deref() {
        Some(key) => Some(
            state
                .storage
                .presigned_get_url(key, AVATAR_SIGNED_URL_TTL)
                .await
                .map_err(ProfileError::Storage)?,
        ),
        None => None,
    };

    Ok(ProfileResponse {
        profile: user.into(),
        avatar_url,
    })
}

fn normalize_profile_update(
    payload: UpdateProfileRequest,
) -> Result<NormalizedProfileUpdate, ProfileError> {
    Ok(NormalizedProfileUpdate {
        display_name: normalize_optional_text(payload.display_name, "display_name", 120)?,
        full_name: normalize_optional_text(payload.full_name, "full_name", 120)?,
        bio: normalize_optional_text(payload.bio, "bio", 500)?,
        location: normalize_optional_text(payload.location, "location", 120)?,
        website_url: normalize_website_url(payload.website_url)?,
    })
}

fn normalize_optional_text(
    value: Option<Option<String>>,
    field: &'static str,
    max_len: usize,
) -> Result<ProfilePatchValue, ProfileError> {
    let Some(value) = value else {
        return Ok(ProfilePatchValue {
            provided: false,
            value: None,
        });
    };
    let Some(value) = value else {
        return Ok(ProfilePatchValue {
            provided: true,
            value: None,
        });
    };
    let value = value.trim();

    if value.is_empty() {
        return Ok(ProfilePatchValue {
            provided: true,
            value: None,
        });
    }
    if value.chars().count() > max_len {
        return Err(ProfileError::BadRequest(anyhow::anyhow!(
            "{field} must be {max_len} characters or fewer"
        )));
    }

    Ok(ProfilePatchValue {
        provided: true,
        value: Some(value.to_owned()),
    })
}

fn normalize_website_url(value: Option<Option<String>>) -> Result<ProfilePatchValue, ProfileError> {
    let value = normalize_optional_text(value, "website_url", 2048)?;
    let Some(url) = value.value.as_deref() else {
        return Ok(value);
    };

    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(ProfileError::BadRequest(anyhow::anyhow!(
            "website_url must start with https:// or http://"
        )));
    }

    Ok(value)
}

async fn extract_profile_photo(
    multipart: &mut Multipart,
) -> Result<ProfilePhotoUpload, ProfileError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(ProfileError::Multipart)?
    {
        let name = field.name().map(str::to_owned);
        if !matches!(
            name.as_deref(),
            Some("photo") | Some("file") | Some("avatar")
        ) {
            continue;
        }

        let content_type = field.content_type().map(str::to_owned).ok_or_else(|| {
            ProfileError::BadRequest(anyhow::anyhow!("photo content type is required"))
        })?;
        let bytes = field.bytes().await.map_err(ProfileError::Multipart)?;
        if bytes.is_empty() {
            return Err(ProfileError::BadRequest(anyhow::anyhow!(
                "photo file must not be empty"
            )));
        }
        if bytes.len() > MAX_PROFILE_PHOTO_BYTES {
            return Err(ProfileError::BadRequest(anyhow::anyhow!(
                "photo file must be 5 MB or smaller"
            )));
        }
        image_extension(&content_type)?;

        return Ok(ProfilePhotoUpload {
            bytes,
            content_type,
        });
    }

    Err(ProfileError::BadRequest(anyhow::anyhow!(
        "multipart form must include a photo file"
    )))
}

fn image_extension(content_type: &str) -> Result<&'static str, ProfileError> {
    match content_type {
        "image/jpeg" => Ok("jpg"),
        "image/png" => Ok("png"),
        "image/webp" => Ok("webp"),
        "image/gif" => Ok("gif"),
        _ => Err(ProfileError::BadRequest(anyhow::anyhow!(
            "photo must be a JPEG, PNG, WEBP, or GIF image"
        ))),
    }
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

#[derive(Debug)]
enum RegisterError {
    BadRequest(anyhow::Error),
    EmailSend(anyhow::Error),
}

#[derive(Debug)]
enum ProfileError {
    BadRequest(anyhow::Error),
    Multipart(axum::extract::multipart::MultipartError),
    Database(sqlx::Error),
    Storage(anyhow::Error),
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

impl IntoResponse for ProfileError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::BadRequest(error) => (StatusCode::BAD_REQUEST, "bad_request", error.to_string()),
            Self::Multipart(error) => (
                StatusCode::BAD_REQUEST,
                "invalid_multipart",
                format!("profile photo upload could not be read: {error}"),
            ),
            Self::Database(error) => {
                tracing::error!(%error, "profile database operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "profile_update_failed",
                    "profile could not be updated".to_owned(),
                )
            }
            Self::Storage(error) => {
                tracing::error!(%error, "profile object storage operation failed");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "profile_photo_storage_unavailable",
                    "profile photo storage is unavailable; try again shortly".to_owned(),
                )
            }
        };

        (status, Json(ErrorResponse { code, message })).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        image_extension, normalize_email, normalize_profile_update, normalize_website_url,
        ProfilePatchValue, UpdateProfileRequest,
    };

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

    #[test]
    fn validates_profile_photo_content_type() {
        assert_eq!(image_extension("image/jpeg").expect("jpeg accepted"), "jpg");
        assert_eq!(image_extension("image/png").expect("png accepted"), "png");
        assert!(image_extension("text/plain").is_err());
    }

    #[test]
    fn validates_profile_website_url() {
        assert_eq!(
            normalize_website_url(Some(Some(" https://example.com/profile ".to_owned())))
                .expect("valid URL"),
            super::ProfilePatchValue {
                provided: true,
                value: Some("https://example.com/profile".to_owned())
            }
        );
        assert!(normalize_website_url(Some(Some("javascript:alert(1)".to_owned()))).is_err());
    }

    #[test]
    fn normalizes_profile_update_fields() {
        let update = normalize_profile_update(UpdateProfileRequest {
            display_name: Some(Some("  Alex  ".to_owned())),
            full_name: Some(Some("   ".to_owned())),
            bio: Some(None),
            location: None,
            website_url: Some(Some(" https://example.com/alex ".to_owned())),
        })
        .expect("profile update should normalize");

        assert_eq!(
            update.display_name,
            ProfilePatchValue {
                provided: true,
                value: Some("Alex".to_owned())
            }
        );
        assert_eq!(
            update.full_name,
            ProfilePatchValue {
                provided: true,
                value: None
            }
        );
        assert_eq!(
            update.bio,
            ProfilePatchValue {
                provided: true,
                value: None
            }
        );
        assert_eq!(
            update.location,
            ProfilePatchValue {
                provided: false,
                value: None
            }
        );
        assert_eq!(
            update.website_url,
            ProfilePatchValue {
                provided: true,
                value: Some("https://example.com/alex".to_owned())
            }
        );
    }

    #[test]
    fn rejects_profile_update_field_over_limits() {
        assert!(normalize_profile_update(UpdateProfileRequest {
            display_name: Some(Some("x".repeat(121))),
            full_name: None,
            bio: None,
            location: None,
            website_url: None,
        })
        .is_err());
    }
}
