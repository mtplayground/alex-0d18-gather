use std::time::Duration;

use axum::{
    extract::{DefaultBodyLimit, Extension, Multipart, Path, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{middleware::require_auth, session::AuthenticatedSession},
    email::{EmailMessage, EmailSendOutcome},
    models::{
        event::Event,
        invitation::{Invitation, INVITATION_STATUS_PENDING},
    },
    state::AppState,
};

const MAX_EVENT_BODY_BYTES: usize = 128 * 1024 * 1024;
const MAX_COVER_IMAGE_BYTES: usize = 8 * 1024 * 1024;
const MAX_PDF_ATTACHMENT_BYTES: usize = 10 * 1024 * 1024;
const MAX_PDF_ATTACHMENTS: usize = 20;
const EVENT_ASSET_URL_TTL: Duration = Duration::from_secs(60 * 60);
const DASHBOARD_EVENT_LIMIT: i64 = 100;

#[derive(Debug, Default)]
struct EventMultipartInput {
    title: Option<String>,
    description: Option<String>,
    starts_at: Option<String>,
    ends_at: Option<String>,
    timezone: Option<String>,
    location_name: Option<String>,
    location_address: Option<String>,
    cover_image: Option<UploadPart>,
    pdf_attachments: Vec<UploadPart>,
}

#[derive(Debug)]
struct UploadPart {
    bytes: Bytes,
    content_type: String,
}

#[derive(Debug)]
struct NormalizedEventInput {
    title: String,
    description: Option<String>,
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
    timezone: Option<String>,
    location_name: Option<String>,
    location_address: Option<String>,
    cover_image: Option<UploadPart>,
    pdf_attachments: Vec<UploadPart>,
}

#[derive(Debug, Serialize)]
struct CreateEventResponse {
    status: &'static str,
    event: Event,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum EventViewerRole {
    Host,
    #[allow(dead_code)]
    Guest,
}

#[derive(Debug, Serialize)]
struct EventAsset {
    object_key: String,
    url: String,
}

#[derive(Debug, Serialize)]
struct EventDetail {
    event: Event,
    viewer_role: EventViewerRole,
    cover_image: Option<EventAsset>,
    pdf_attachments: Vec<EventAsset>,
}

#[derive(Debug, Serialize)]
struct EventDetailResponse {
    event: EventDetail,
}

#[derive(Debug, Serialize)]
struct DashboardEvent {
    id: Uuid,
    title: String,
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
    timezone: Option<String>,
    location_name: Option<String>,
    cover_image: Option<EventAsset>,
    viewer_role: EventViewerRole,
}

#[derive(Debug, Serialize)]
struct DashboardEventsResponse {
    upcoming: Vec<DashboardEvent>,
    past: Vec<DashboardEvent>,
}

#[derive(Debug, Deserialize)]
struct CreateInvitationRequest {
    email: Option<String>,
    invitee_email: Option<String>,
    invitee_user_id: Option<Uuid>,
}

#[derive(Debug)]
struct NormalizedInvitationInput {
    invitee_email: String,
    invitee_user_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
struct CreateInvitationResponse {
    status: &'static str,
    invitation: Invitation,
    invitation_url: String,
    email_sent: bool,
}

#[derive(Debug, Serialize)]
struct EventErrorResponse {
    code: &'static str,
    message: String,
}

#[derive(Debug)]
enum EventError {
    BadRequest(anyhow::Error),
    NotFound,
    Multipart(axum::extract::multipart::MultipartError),
    Storage(anyhow::Error),
    Email(anyhow::Error),
    Database(sqlx::Error),
}

pub fn protected_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(list_dashboard_events))
        .route("/:event_id/invitations", post(create_invitation))
        .route("/:event_id", get(get_event_detail))
        .route("/", post(create_event))
        .layer(DefaultBodyLimit::max(MAX_EVENT_BODY_BYTES))
        .route_layer(middleware::from_fn_with_state(state, require_auth))
}

async fn get_event_detail(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<EventDetailResponse>, EventError> {
    let event = fetch_hosted_event(&state, session.user.id, event_id)
        .await?
        .ok_or(EventError::NotFound)?;
    let event = event_detail(&state, event).await?;

    Ok(Json(EventDetailResponse { event }))
}

async fn list_dashboard_events(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
) -> Result<Json<DashboardEventsResponse>, EventError> {
    let upcoming =
        fetch_dashboard_events(&state, session.user.id, DashboardBucket::Upcoming).await?;
    let past = fetch_dashboard_events(&state, session.user.id, DashboardBucket::Past).await?;

    Ok(Json(DashboardEventsResponse {
        upcoming: dashboard_events(&state, upcoming).await?,
        past: dashboard_events(&state, past).await?,
    }))
}

async fn create_invitation(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<CreateInvitationRequest>,
) -> Result<(StatusCode, Json<CreateInvitationResponse>), EventError> {
    let event = fetch_hosted_event(&state, session.user.id, event_id)
        .await?
        .ok_or(EventError::NotFound)?;
    let input = normalize_invitation_input(&state, payload).await?;
    let invitation = insert_invitation(
        &state,
        event.id,
        input.invitee_user_id,
        input.invitee_email.clone(),
    )
    .await
    .map_err(invitation_insert_error)?;
    let invitation_path = format!("/invitations/{}", invitation.share_token);
    let invitation_url = state
        .auth_links
        .login_url(Some(&invitation_path))
        .map_err(EventError::BadRequest)?;

    let message = invitation_message(&event, &input.invitee_email, &invitation_url);
    let email_sent = match state.email.send(message).await {
        Ok(EmailSendOutcome::Sent { message_id }) => {
            tracing::info!(
                %message_id,
                event_id = %event.id,
                invitation_id = %invitation.id,
                invitee_email = %input.invitee_email,
                "event invitation email sent"
            );
            true
        }
        Ok(EmailSendOutcome::Skipped { reason }) => {
            tracing::warn!(
                %reason,
                event_id = %event.id,
                invitation_id = %invitation.id,
                invitee_email = %input.invitee_email,
                "event invitation email skipped"
            );
            false
        }
        Err(error) => {
            cleanup_invitation_after_email_failure(&state, invitation.id).await;
            return Err(EventError::Email(error));
        }
    };

    Ok((
        StatusCode::CREATED,
        Json(CreateInvitationResponse {
            status: if email_sent {
                "invitation_email_sent"
            } else {
                "invitation_created"
            },
            invitation,
            invitation_url,
            email_sent,
        }),
    ))
}

async fn create_event(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<CreateEventResponse>), EventError> {
    let input = normalize_event_input(read_event_multipart(multipart).await?)?;
    let event_id = Uuid::new_v4();
    let organizer_id = session.user.id;
    let mut uploaded_keys = Vec::new();

    let cover_image_object_key = match input.cover_image {
        Some(cover_image) => {
            let extension = cover_image_extension(&cover_image.content_type)?;
            let relative_key = format!(
                "events/{organizer_id}/{event_id}/cover/{}.{}",
                Uuid::new_v4(),
                extension
            );
            let stored = match upload_event_object(&state, &relative_key, cover_image).await {
                Ok(stored) => stored,
                Err(error) => {
                    cleanup_uploaded_objects(&state, &uploaded_keys).await;
                    return Err(error);
                }
            };
            uploaded_keys.push(stored.clone());
            Some(stored)
        }
        None => None,
    };

    let mut pdf_attachment_object_keys = Vec::with_capacity(input.pdf_attachments.len());
    for attachment in input.pdf_attachments {
        let relative_key = format!(
            "events/{organizer_id}/{event_id}/attachments/{}.pdf",
            Uuid::new_v4()
        );
        let stored = match upload_event_object(&state, &relative_key, attachment).await {
            Ok(stored) => stored,
            Err(error) => {
                cleanup_uploaded_objects(&state, &uploaded_keys).await;
                return Err(error);
            }
        };
        uploaded_keys.push(stored.clone());
        pdf_attachment_object_keys.push(stored);
    }

    let event = match insert_event(
        &state,
        event_id,
        organizer_id,
        input.title,
        input.description,
        input.starts_at,
        input.ends_at,
        input.timezone,
        input.location_name,
        input.location_address,
        cover_image_object_key,
        pdf_attachment_object_keys,
    )
    .await
    {
        Ok(event) => event,
        Err(error) => {
            cleanup_uploaded_objects(&state, &uploaded_keys).await;
            return Err(EventError::Database(error));
        }
    };

    Ok((
        StatusCode::CREATED,
        Json(CreateEventResponse {
            status: "event_created",
            event,
        }),
    ))
}

async fn insert_event(
    state: &AppState,
    event_id: Uuid,
    organizer_id: Uuid,
    title: String,
    description: Option<String>,
    starts_at: DateTime<Utc>,
    ends_at: Option<DateTime<Utc>>,
    timezone: Option<String>,
    location_name: Option<String>,
    location_address: Option<String>,
    cover_image_object_key: Option<String>,
    pdf_attachment_object_keys: Vec<String>,
) -> Result<Event, sqlx::Error> {
    sqlx::query_as::<_, Event>(
        r#"
        INSERT INTO events (
            id,
            organizer_user_id,
            title,
            description,
            starts_at,
            ends_at,
            timezone,
            location_name,
            location_address,
            cover_image_object_key,
            pdf_attachment_object_keys
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
            id,
            organizer_user_id,
            title,
            description,
            starts_at,
            ends_at,
            timezone,
            location_name,
            location_address,
            cover_image_object_key,
            pdf_attachment_object_keys,
            created_at,
            updated_at
        "#,
    )
    .bind(event_id)
    .bind(organizer_id)
    .bind(title)
    .bind(description)
    .bind(starts_at)
    .bind(ends_at)
    .bind(timezone)
    .bind(location_name)
    .bind(location_address)
    .bind(cover_image_object_key)
    .bind(pdf_attachment_object_keys)
    .fetch_one(&state.db)
    .await
}

async fn insert_invitation(
    state: &AppState,
    event_id: Uuid,
    invitee_user_id: Option<Uuid>,
    invitee_email: String,
) -> Result<Invitation, sqlx::Error> {
    sqlx::query_as::<_, Invitation>(
        r#"
        INSERT INTO invitations (
            event_id,
            invitee_user_id,
            invitee_email,
            status
        )
        VALUES ($1, $2, $3, $4)
        RETURNING
            id,
            event_id,
            invitee_user_id,
            invitee_email,
            status,
            share_token,
            created_at,
            updated_at
        "#,
    )
    .bind(event_id)
    .bind(invitee_user_id)
    .bind(invitee_email)
    .bind(INVITATION_STATUS_PENDING)
    .fetch_one(&state.db)
    .await
}

async fn cleanup_invitation_after_email_failure(state: &AppState, invitation_id: Uuid) {
    if let Err(error) = sqlx::query(
        r#"
        DELETE FROM invitations
        WHERE id = $1
        "#,
    )
    .bind(invitation_id)
    .execute(&state.db)
    .await
    {
        tracing::warn!(
            %error,
            %invitation_id,
            "failed to clean up invitation after email send failure"
        );
    }
}

async fn fetch_hosted_event(
    state: &AppState,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<Option<Event>, EventError> {
    sqlx::query_as::<_, Event>(
        r#"
        SELECT
            id,
            organizer_user_id,
            title,
            description,
            starts_at,
            ends_at,
            timezone,
            location_name,
            location_address,
            cover_image_object_key,
            pdf_attachment_object_keys,
            created_at,
            updated_at
        FROM events
        WHERE id = $1 AND organizer_user_id = $2
        "#,
    )
    .bind(event_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(EventError::Database)
}

#[derive(Debug, Clone, Copy)]
enum DashboardBucket {
    Upcoming,
    Past,
}

async fn fetch_dashboard_events(
    state: &AppState,
    user_id: Uuid,
    bucket: DashboardBucket,
) -> Result<Vec<Event>, EventError> {
    let timing_predicate = match bucket {
        DashboardBucket::Upcoming => "starts_at >= NOW()",
        DashboardBucket::Past => "starts_at < NOW()",
    };
    let sort_direction = match bucket {
        DashboardBucket::Upcoming => "ASC",
        DashboardBucket::Past => "DESC",
    };
    let query = format!(
        r#"
        SELECT
            id,
            organizer_user_id,
            title,
            description,
            starts_at,
            ends_at,
            timezone,
            location_name,
            location_address,
            cover_image_object_key,
            pdf_attachment_object_keys,
            created_at,
            updated_at
        FROM events
        WHERE organizer_user_id = $1 AND {timing_predicate}
        ORDER BY starts_at {sort_direction}, created_at DESC
        LIMIT $2
        "#
    );

    sqlx::query_as::<_, Event>(&query)
        .bind(user_id)
        .bind(DASHBOARD_EVENT_LIMIT)
        .fetch_all(&state.db)
        .await
        .map_err(EventError::Database)
}

async fn event_detail(state: &AppState, event: Event) -> Result<EventDetail, EventError> {
    let cover_image = signed_asset(state, event.cover_image_object_key.as_deref()).await?;
    let pdf_attachments = signed_assets(state, &event.pdf_attachment_object_keys).await?;

    Ok(EventDetail {
        event,
        viewer_role: EventViewerRole::Host,
        cover_image,
        pdf_attachments,
    })
}

async fn dashboard_events(
    state: &AppState,
    events: Vec<Event>,
) -> Result<Vec<DashboardEvent>, EventError> {
    let mut dashboard_events = Vec::with_capacity(events.len());
    for event in events {
        let cover_image = signed_asset(state, event.cover_image_object_key.as_deref()).await?;
        dashboard_events.push(DashboardEvent {
            id: event.id,
            title: event.title,
            starts_at: event.starts_at,
            ends_at: event.ends_at,
            timezone: event.timezone,
            location_name: event.location_name,
            cover_image,
            viewer_role: EventViewerRole::Host,
        });
    }

    Ok(dashboard_events)
}

async fn signed_assets(
    state: &AppState,
    object_keys: &[String],
) -> Result<Vec<EventAsset>, EventError> {
    let mut assets = Vec::with_capacity(object_keys.len());
    for object_key in object_keys {
        if let Some(asset) = signed_asset(state, Some(object_key)).await? {
            assets.push(asset);
        }
    }

    Ok(assets)
}

async fn signed_asset(
    state: &AppState,
    object_key: Option<&str>,
) -> Result<Option<EventAsset>, EventError> {
    let Some(object_key) = object_key else {
        return Ok(None);
    };
    let url = state
        .storage
        .presigned_get_url(object_key, EVENT_ASSET_URL_TTL)
        .await
        .map_err(EventError::Storage)?;

    Ok(Some(EventAsset {
        object_key: object_key.to_owned(),
        url,
    }))
}

async fn read_event_multipart(mut multipart: Multipart) -> Result<EventMultipartInput, EventError> {
    let mut input = EventMultipartInput::default();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(EventError::Multipart)?
    {
        let name = field.name().map(str::to_owned).unwrap_or_default();

        match name.as_str() {
            "title" => input.title = Some(read_text_field(field).await?),
            "description" => input.description = Some(read_text_field(field).await?),
            "starts_at" => input.starts_at = Some(read_text_field(field).await?),
            "ends_at" => input.ends_at = Some(read_text_field(field).await?),
            "timezone" => input.timezone = Some(read_text_field(field).await?),
            "location_name" => input.location_name = Some(read_text_field(field).await?),
            "location_address" => input.location_address = Some(read_text_field(field).await?),
            "cover_image" | "cover" => {
                if input.cover_image.is_some() {
                    return Err(EventError::BadRequest(anyhow::anyhow!(
                        "only one cover image may be uploaded"
                    )));
                }
                input.cover_image = Some(read_upload_part(field, MAX_COVER_IMAGE_BYTES).await?);
            }
            "pdf_attachments" | "pdf_attachment" | "pdfs" | "attachments" => {
                if input.pdf_attachments.len() >= MAX_PDF_ATTACHMENTS {
                    return Err(EventError::BadRequest(anyhow::anyhow!(
                        "at most {MAX_PDF_ATTACHMENTS} PDF attachments may be uploaded"
                    )));
                }
                input
                    .pdf_attachments
                    .push(read_upload_part(field, MAX_PDF_ATTACHMENT_BYTES).await?);
            }
            _ => {}
        }
    }

    Ok(input)
}

async fn read_text_field(field: axum::extract::multipart::Field<'_>) -> Result<String, EventError> {
    field.text().await.map_err(EventError::Multipart)
}

async fn read_upload_part(
    field: axum::extract::multipart::Field<'_>,
    max_bytes: usize,
) -> Result<UploadPart, EventError> {
    let content_type = field
        .content_type()
        .map(str::to_owned)
        .ok_or_else(|| EventError::BadRequest(anyhow::anyhow!("file content type is required")))?;
    let bytes = field.bytes().await.map_err(EventError::Multipart)?;

    if bytes.is_empty() {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "uploaded file must not be empty"
        )));
    }
    if bytes.len() > max_bytes {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "uploaded file exceeds the allowed size"
        )));
    }

    Ok(UploadPart {
        bytes,
        content_type,
    })
}

async fn normalize_invitation_input(
    state: &AppState,
    payload: CreateInvitationRequest,
) -> Result<NormalizedInvitationInput, EventError> {
    let invitee_user_id = payload.invitee_user_id;
    let requested_email = payload
        .email
        .or(payload.invitee_email)
        .map(|value| normalize_email(&value))
        .transpose()?;
    let invitee_email = match (requested_email, invitee_user_id) {
        (Some(email), _) => email,
        (None, Some(user_id)) => fetch_user_email(state, user_id)
            .await?
            .ok_or_else(|| EventError::BadRequest(anyhow::anyhow!("invitee user was not found")))?,
        (None, None) => {
            return Err(EventError::BadRequest(anyhow::anyhow!(
                "invitee email or user id is required"
            )));
        }
    };

    Ok(NormalizedInvitationInput {
        invitee_email,
        invitee_user_id,
    })
}

async fn fetch_user_email(state: &AppState, user_id: Uuid) -> Result<Option<String>, EventError> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT email
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(EventError::Database)
    .and_then(|email| email.map(|email| normalize_email(&email)).transpose())
}

fn normalize_email(value: &str) -> Result<String, EventError> {
    let email = value.trim().to_ascii_lowercase();
    if email.is_empty() {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "invitee email is required"
        )));
    }
    if email.len() > 320 {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "invitee email must be 320 characters or fewer"
        )));
    }
    if !email.contains('@') {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "invitee email must include @"
        )));
    }

    Ok(email)
}

fn invitation_insert_error(error: sqlx::Error) -> EventError {
    if database_error_code_is(&error, "23505") {
        return EventError::BadRequest(anyhow::anyhow!(
            "an invitation already exists for this event and invitee"
        ));
    }
    if database_error_code_is(&error, "23503") {
        return EventError::BadRequest(anyhow::anyhow!("invitee user was not found"));
    }

    EventError::Database(error)
}

fn database_error_code_is(error: &sqlx::Error, code: &str) -> bool {
    error
        .as_database_error()
        .and_then(|database_error| database_error.code())
        .is_some_and(|database_code| database_code == code)
}

fn invitation_message(event: &Event, invitee_email: &str, invitation_url: &str) -> EmailMessage {
    let title = escape_html(&event.title);
    let escaped_url = escape_html(invitation_url);
    let escaped_email = escape_html(invitee_email);
    let starts_at = event.starts_at.to_rfc3339();
    let location = event
        .location_name
        .as_deref()
        .unwrap_or("Location to be announced");
    let escaped_location = escape_html(location);

    EmailMessage {
        to: vec![invitee_email.to_owned()],
        subject: format!("Invitation: {}", event.title),
        html: Some(format!(
            r#"<p>Hello {escaped_email},</p>
<p>You have been invited to <strong>{title}</strong>.</p>
<p><strong>When:</strong> {starts_at}<br><strong>Where:</strong> {escaped_location}</p>
<p><a href="{escaped_url}">View your invitation</a></p>"#
        )),
        text: Some(format!(
            "You have been invited to {}.\n\nWhen: {}\nWhere: {}\n\nView your invitation: {}",
            event.title, starts_at, location, invitation_url
        )),
        reply_to: None,
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn normalize_event_input(input: EventMultipartInput) -> Result<NormalizedEventInput, EventError> {
    let title = normalize_required_text(input.title, "title", 200)?;
    let description = normalize_optional_text(input.description, "description", 5000)?;
    let starts_at = parse_required_datetime(input.starts_at, "starts_at")?;
    let ends_at = parse_optional_datetime(input.ends_at, "ends_at")?;

    if ends_at.is_some_and(|ends_at| ends_at <= starts_at) {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "ends_at must be later than starts_at"
        )));
    }

    let timezone = normalize_optional_text(input.timezone, "timezone", 100)?;
    let location_name = normalize_optional_text(input.location_name, "location_name", 200)?;
    let location_address =
        normalize_optional_text(input.location_address, "location_address", 500)?;

    if let Some(cover_image) = input.cover_image.as_ref() {
        cover_image_extension(&cover_image.content_type)?;
    }
    for attachment in &input.pdf_attachments {
        validate_pdf_content_type(&attachment.content_type)?;
    }

    Ok(NormalizedEventInput {
        title,
        description,
        starts_at,
        ends_at,
        timezone,
        location_name,
        location_address,
        cover_image: input.cover_image,
        pdf_attachments: input.pdf_attachments,
    })
}

fn normalize_required_text(
    value: Option<String>,
    field: &'static str,
    max_len: usize,
) -> Result<String, EventError> {
    let value = normalize_optional_text(value, field, max_len)?;
    value.ok_or_else(|| EventError::BadRequest(anyhow::anyhow!("{field} is required")))
}

fn normalize_optional_text(
    value: Option<String>,
    field: &'static str,
    max_len: usize,
) -> Result<Option<String>, EventError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if value.chars().count() > max_len {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "{field} must be {max_len} characters or fewer"
        )));
    }

    Ok(Some(value.to_owned()))
}

fn parse_required_datetime(
    value: Option<String>,
    field: &'static str,
) -> Result<DateTime<Utc>, EventError> {
    parse_optional_datetime(value, field)?
        .ok_or_else(|| EventError::BadRequest(anyhow::anyhow!("{field} is required")))
}

fn parse_optional_datetime(
    value: Option<String>,
    field: &'static str,
) -> Result<Option<DateTime<Utc>>, EventError> {
    let Some(value) = normalize_optional_text(value, field, 64)? else {
        return Ok(None);
    };

    DateTime::parse_from_rfc3339(&value)
        .map(|value| Some(value.with_timezone(&Utc)))
        .map_err(|error| {
            EventError::BadRequest(anyhow::anyhow!("{field} must be RFC3339: {error}"))
        })
}

fn cover_image_extension(content_type: &str) -> Result<&'static str, EventError> {
    match content_type {
        "image/jpeg" => Ok("jpg"),
        "image/png" => Ok("png"),
        "image/webp" => Ok("webp"),
        "image/gif" => Ok("gif"),
        _ => Err(EventError::BadRequest(anyhow::anyhow!(
            "cover image must be a JPEG, PNG, WEBP, or GIF image"
        ))),
    }
}

fn validate_pdf_content_type(content_type: &str) -> Result<(), EventError> {
    if content_type == "application/pdf" {
        return Ok(());
    }

    Err(EventError::BadRequest(anyhow::anyhow!(
        "attachments must be PDF files"
    )))
}

async fn upload_event_object(
    state: &AppState,
    relative_key: &str,
    upload: UploadPart,
) -> Result<String, EventError> {
    state
        .storage
        .put_object(
            relative_key,
            upload.bytes,
            Some(upload.content_type.as_str()),
        )
        .await
        .map(|stored| stored.relative_key)
        .map_err(EventError::Storage)
}

async fn cleanup_uploaded_objects(state: &AppState, relative_keys: &[String]) {
    for key in relative_keys {
        if let Err(error) = state.storage.delete_object(key).await {
            tracing::warn!(%error, object_key = %key, "failed to clean up event upload after create failure");
        }
    }
}

impl IntoResponse for EventError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::BadRequest(error) => (StatusCode::BAD_REQUEST, "bad_request", error.to_string()),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "event_not_found",
                "event was not found".to_owned(),
            ),
            Self::Multipart(error) => (
                StatusCode::BAD_REQUEST,
                "invalid_multipart",
                format!("event upload could not be read: {error}"),
            ),
            Self::Storage(error) => {
                tracing::error!(%error, "event object storage operation failed");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "event_storage_unavailable",
                    "event files could not be stored; try again shortly".to_owned(),
                )
            }
            Self::Email(error) => {
                tracing::error!(%error, "event invitation email send failed");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "event_invitation_email_unavailable",
                    "invitation email could not be sent; try again shortly".to_owned(),
                )
            }
            Self::Database(error) => {
                tracing::error!(%error, "event database operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "event_operation_failed",
                    "event operation could not be completed".to_owned(),
                )
            }
        };

        (status, Json(EventErrorResponse { code, message })).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cover_image_extension, escape_html, normalize_email, parse_required_datetime,
        validate_pdf_content_type,
    };

    #[test]
    fn validates_event_upload_content_types() {
        assert_eq!(
            cover_image_extension("image/jpeg").expect("jpeg accepted"),
            "jpg"
        );
        assert!(cover_image_extension("application/pdf").is_err());
        assert!(validate_pdf_content_type("application/pdf").is_ok());
        assert!(validate_pdf_content_type("image/png").is_err());
    }

    #[test]
    fn parses_rfc3339_datetime() {
        let parsed = parse_required_datetime(Some("2026-08-01T18:00:00Z".to_owned()), "starts_at")
            .expect("valid timestamp");
        assert_eq!(parsed.to_rfc3339(), "2026-08-01T18:00:00+00:00");
        assert!(parse_required_datetime(Some("tomorrow".to_owned()), "starts_at").is_err());
    }

    #[test]
    fn normalizes_invitation_email() {
        assert_eq!(
            normalize_email("  Friend@Example.COM ").expect("email accepted"),
            "friend@example.com"
        );
        assert!(normalize_email("missing-at").is_err());
    }

    #[test]
    fn escapes_invitation_email_html() {
        assert_eq!(
            escape_html("<Gather & friends>"),
            "&lt;Gather &amp; friends&gt;"
        );
    }
}
