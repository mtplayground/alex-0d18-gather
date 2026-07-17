use std::time::Duration;

use axum::{
    extract::{DefaultBodyLimit, Extension, Multipart, Path, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{middleware::require_auth, session::AuthenticatedSession},
    email::{
        templates::{self, EventInvitationTemplate, RsvpConfirmationTemplate},
        EmailMessage, EmailSendOutcome,
    },
    models::{
        activity::{
            EventActivityEntry, ACTIVITY_COMMENT_CREATED, ACTIVITY_EVENT_CREATED,
            ACTIVITY_RSVP_UPDATED,
        },
        comment::EventComment,
        event::Event,
        invitation::{
            Invitation, INVITATION_STATUS_ACCEPTED, INVITATION_STATUS_DECLINED,
            INVITATION_STATUS_PENDING, RSVP_STATUS_MAYBE, RSVP_STATUS_NO, RSVP_STATUS_YES,
        },
    },
    state::AppState,
};

const MAX_EVENT_BODY_BYTES: usize = 128 * 1024 * 1024;
const MAX_COVER_IMAGE_BYTES: usize = 8 * 1024 * 1024;
const MAX_PDF_ATTACHMENT_BYTES: usize = 10 * 1024 * 1024;
const MAX_PDF_ATTACHMENTS: usize = 20;
const MAX_COMMENT_BODY_CHARS: usize = 2000;
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
struct InvitationLinkResponse {
    status: &'static str,
    invitation: Invitation,
    invitation_url: String,
}

#[derive(Debug, Serialize)]
struct AcceptInvitationResponse {
    status: &'static str,
    invitation: Invitation,
    event: Event,
}

#[derive(Debug, Deserialize)]
struct RsvpRequest {
    status: String,
}

#[derive(Debug, Serialize)]
struct RsvpResponse {
    status: &'static str,
    invitation: Invitation,
    event: Event,
    email_sent: bool,
}

#[derive(Debug, Clone, Serialize)]
struct RsvpListEntry {
    invitation_id: Uuid,
    invitee_user_id: Option<Uuid>,
    invitee_email: Option<String>,
    rsvp_status: String,
    rsvp_responded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct EventRsvpListResponse {
    event_id: Uuid,
    coming: Vec<RsvpListEntry>,
    declined: Vec<RsvpListEntry>,
    maybe: Vec<RsvpListEntry>,
}

#[derive(Debug, Deserialize)]
struct CreateCommentRequest {
    body: String,
}

#[derive(Debug, Serialize)]
struct CommentListResponse {
    comments: Vec<EventComment>,
}

#[derive(Debug, Serialize)]
struct CreateCommentResponse {
    status: &'static str,
    comment: EventComment,
}

#[derive(Debug, Serialize)]
struct DeleteCommentResponse {
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct EventTimelineResponse {
    activity: Vec<EventActivityEntry>,
}

#[derive(Debug)]
enum EventError {
    BadRequest(anyhow::Error),
    Forbidden(anyhow::Error),
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
        .route(
            "/:event_id/invitations/share-link",
            post(create_invitation_share_link),
        )
        .route(
            "/:event_id/comments",
            get(list_comments).post(create_comment),
        )
        .route("/:event_id/comments/:comment_id", delete(delete_comment))
        .route("/:event_id/timeline", get(list_event_timeline))
        .route("/:event_id/rsvps", get(list_event_rsvps))
        .route("/:event_id", get(get_event_detail))
        .route("/", post(create_event))
        .layer(DefaultBodyLimit::max(MAX_EVENT_BODY_BYTES))
        .route_layer(middleware::from_fn_with_state(state, require_auth))
}

pub fn invitation_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/:share_token/accept", post(accept_invitation))
        .route("/:share_token/rsvp", post(update_rsvp))
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

async fn list_event_rsvps(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<EventRsvpListResponse>, EventError> {
    let event = fetch_hosted_event(&state, session.user.id, event_id)
        .await?
        .ok_or(EventError::NotFound)?;
    let invitations = fetch_event_rsvp_invitations(&state, event.id).await?;

    Ok(Json(rsvp_list_response(event.id, invitations)))
}

async fn list_comments(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<CommentListResponse>, EventError> {
    fetch_commentable_event(
        &state,
        session.user.id,
        session.user.email.as_str(),
        event_id,
    )
    .await?
    .ok_or(EventError::NotFound)?;
    let comments = fetch_event_comments(&state, event_id).await?;

    Ok(Json(CommentListResponse { comments }))
}

async fn create_comment(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<CreateCommentResponse>), EventError> {
    let event = fetch_commentable_event(
        &state,
        session.user.id,
        session.user.email.as_str(),
        event_id,
    )
    .await?
    .ok_or(EventError::NotFound)?;
    let body = normalize_comment_body(&payload.body)?;
    let comment = insert_event_comment(&state, event.id, session.user.id, body).await?;
    insert_event_activity(
        &state,
        event.id,
        Some(session.user.id),
        ACTIVITY_COMMENT_CREATED,
        Some("comment"),
        Some(comment.id),
        "Comment added",
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateCommentResponse {
            status: "comment_created",
            comment,
        }),
    ))
}

async fn delete_comment(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path((event_id, comment_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DeleteCommentResponse>, EventError> {
    let deleted = delete_event_comment(&state, event_id, comment_id, session.user.id).await?;
    if !deleted {
        return Err(EventError::NotFound);
    }

    Ok(Json(DeleteCommentResponse {
        status: "comment_deleted",
    }))
}

async fn list_event_timeline(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<EventTimelineResponse>, EventError> {
    fetch_commentable_event(
        &state,
        session.user.id,
        session.user.email.as_str(),
        event_id,
    )
    .await?
    .ok_or(EventError::NotFound)?;
    let activity = fetch_event_activity(&state, event_id).await?;

    Ok(Json(EventTimelineResponse { activity }))
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
    let invitation_url = invitation_url(&state, &invitation)?;

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

async fn create_invitation_share_link(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<CreateInvitationRequest>,
) -> Result<(StatusCode, Json<InvitationLinkResponse>), EventError> {
    let event = fetch_hosted_event(&state, session.user.id, event_id)
        .await?
        .ok_or(EventError::NotFound)?;
    let input = normalize_invitation_input(&state, payload).await?;
    let invitation =
        insert_invitation(&state, event.id, input.invitee_user_id, input.invitee_email)
            .await
            .map_err(invitation_insert_error)?;
    let invitation_url = invitation_url(&state, &invitation)?;

    Ok((
        StatusCode::CREATED,
        Json(InvitationLinkResponse {
            status: "invitation_link_created",
            invitation,
            invitation_url,
        }),
    ))
}

async fn accept_invitation(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(share_token): Path<String>,
) -> Result<Json<AcceptInvitationResponse>, EventError> {
    let share_token = normalize_share_token(&share_token)?;
    let invitation = fetch_invitation_by_token(&state, &share_token)
        .await?
        .ok_or(EventError::NotFound)?;

    if invitation.status == "revoked" {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "invitation is no longer active"
        )));
    }
    if invitation
        .invitee_user_id
        .is_some_and(|invitee_user_id| invitee_user_id != session.user.id)
    {
        return Err(EventError::Forbidden(anyhow::anyhow!(
            "invitation belongs to another user"
        )));
    }

    let invitation = accept_invitation_by_token(
        &state,
        &share_token,
        session.user.id,
        session.user.email.clone(),
    )
    .await?;
    let event = fetch_event_by_id(&state, invitation.event_id)
        .await?
        .ok_or(EventError::NotFound)?;

    Ok(Json(AcceptInvitationResponse {
        status: "invitation_accepted",
        invitation,
        event,
    }))
}

async fn update_rsvp(
    State(state): State<AppState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(share_token): Path<String>,
    Json(payload): Json<RsvpRequest>,
) -> Result<Json<RsvpResponse>, EventError> {
    let share_token = normalize_share_token(&share_token)?;
    let rsvp_status = normalize_rsvp_status(&payload.status)?;
    let invitation = fetch_invitation_by_token(&state, &share_token)
        .await?
        .ok_or(EventError::NotFound)?;

    if invitation.status == "revoked" {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "invitation is no longer active"
        )));
    }
    if invitation
        .invitee_user_id
        .is_some_and(|invitee_user_id| invitee_user_id != session.user.id)
    {
        return Err(EventError::Forbidden(anyhow::anyhow!(
            "invitation belongs to another user"
        )));
    }

    let invitation = update_invitation_rsvp(
        &state,
        &share_token,
        &rsvp_status,
        session.user.id,
        session.user.email.clone(),
    )
    .await?;
    let event = fetch_event_by_id(&state, invitation.event_id)
        .await?
        .ok_or(EventError::NotFound)?;
    insert_event_activity(
        &state,
        event.id,
        Some(session.user.id),
        ACTIVITY_RSVP_UPDATED,
        Some("invitation"),
        Some(invitation.id),
        format!("RSVP updated to {}", rsvp_status_label(&rsvp_status)),
    )
    .await?;
    let email_sent =
        send_rsvp_confirmation(&state, &event, &invitation, session.user.email.as_str()).await;

    Ok(Json(RsvpResponse {
        status: "rsvp_updated",
        invitation,
        event,
        email_sent,
    }))
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
    insert_event_activity(
        &state,
        event.id,
        Some(session.user.id),
        ACTIVITY_EVENT_CREATED,
        Some("event"),
        Some(event.id),
        "Event created",
    )
    .await?;

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
            rsvp_status,
            rsvp_responded_at,
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

async fn fetch_invitation_by_token(
    state: &AppState,
    share_token: &str,
) -> Result<Option<Invitation>, EventError> {
    sqlx::query_as::<_, Invitation>(
        r#"
        SELECT
            id,
            event_id,
            invitee_user_id,
            invitee_email,
            status,
            share_token,
            rsvp_status,
            rsvp_responded_at,
            created_at,
            updated_at
        FROM invitations
        WHERE share_token = $1
        "#,
    )
    .bind(share_token)
    .fetch_optional(&state.db)
    .await
    .map_err(EventError::Database)
}

async fn accept_invitation_by_token(
    state: &AppState,
    share_token: &str,
    user_id: Uuid,
    user_email: String,
) -> Result<Invitation, EventError> {
    sqlx::query_as::<_, Invitation>(
        r#"
        UPDATE invitations
        SET
            status = $2,
            invitee_user_id = COALESCE(invitee_user_id, $3),
            invitee_email = COALESCE(invitee_email, $4),
            updated_at = NOW()
        WHERE share_token = $1
            AND status <> 'revoked'
            AND (invitee_user_id IS NULL OR invitee_user_id = $3)
        RETURNING
            id,
            event_id,
            invitee_user_id,
            invitee_email,
            status,
            share_token,
            rsvp_status,
            rsvp_responded_at,
            created_at,
            updated_at
        "#,
    )
    .bind(share_token)
    .bind(INVITATION_STATUS_ACCEPTED)
    .bind(user_id)
    .bind(user_email)
    .fetch_optional(&state.db)
    .await
    .map_err(EventError::Database)?
    .ok_or(EventError::NotFound)
}

async fn update_invitation_rsvp(
    state: &AppState,
    share_token: &str,
    rsvp_status: &str,
    user_id: Uuid,
    user_email: String,
) -> Result<Invitation, EventError> {
    let invitation_status = match rsvp_status {
        RSVP_STATUS_NO => INVITATION_STATUS_DECLINED,
        RSVP_STATUS_YES | RSVP_STATUS_MAYBE => INVITATION_STATUS_ACCEPTED,
        _ => INVITATION_STATUS_PENDING,
    };

    sqlx::query_as::<_, Invitation>(
        r#"
        UPDATE invitations
        SET
            status = $2,
            invitee_user_id = COALESCE(invitee_user_id, $3),
            invitee_email = COALESCE(invitee_email, $4),
            rsvp_status = $5,
            rsvp_responded_at = NOW(),
            updated_at = NOW()
        WHERE share_token = $1
            AND status <> 'revoked'
            AND (invitee_user_id IS NULL OR invitee_user_id = $3)
        RETURNING
            id,
            event_id,
            invitee_user_id,
            invitee_email,
            status,
            share_token,
            rsvp_status,
            rsvp_responded_at,
            created_at,
            updated_at
        "#,
    )
    .bind(share_token)
    .bind(invitation_status)
    .bind(user_id)
    .bind(user_email)
    .bind(rsvp_status)
    .fetch_optional(&state.db)
    .await
    .map_err(EventError::Database)?
    .ok_or(EventError::NotFound)
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

async fn fetch_event_by_id(state: &AppState, event_id: Uuid) -> Result<Option<Event>, EventError> {
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
        WHERE id = $1
        "#,
    )
    .bind(event_id)
    .fetch_optional(&state.db)
    .await
    .map_err(EventError::Database)
}

async fn fetch_commentable_event(
    state: &AppState,
    user_id: Uuid,
    user_email: &str,
    event_id: Uuid,
) -> Result<Option<Event>, EventError> {
    sqlx::query_as::<_, Event>(
        r#"
        SELECT
            e.id,
            e.organizer_user_id,
            e.title,
            e.description,
            e.starts_at,
            e.ends_at,
            e.timezone,
            e.location_name,
            e.location_address,
            e.cover_image_object_key,
            e.pdf_attachment_object_keys,
            e.created_at,
            e.updated_at
        FROM events e
        WHERE e.id = $1
            AND (
                e.organizer_user_id = $2
                OR EXISTS (
                    SELECT 1
                    FROM invitations i
                    WHERE i.event_id = e.id
                        AND i.status = 'accepted'
                        AND (
                            i.invitee_user_id = $2
                            OR lower(i.invitee_email) = lower($3)
                        )
                )
            )
        "#,
    )
    .bind(event_id)
    .bind(user_id)
    .bind(user_email)
    .fetch_optional(&state.db)
    .await
    .map_err(EventError::Database)
}

async fn fetch_event_comments(
    state: &AppState,
    event_id: Uuid,
) -> Result<Vec<EventComment>, EventError> {
    sqlx::query_as::<_, EventComment>(
        r#"
        SELECT
            id,
            event_id,
            author_user_id,
            body,
            created_at,
            updated_at
        FROM event_comments
        WHERE event_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(event_id)
    .fetch_all(&state.db)
    .await
    .map_err(EventError::Database)
}

async fn insert_event_comment(
    state: &AppState,
    event_id: Uuid,
    author_user_id: Uuid,
    body: String,
) -> Result<EventComment, EventError> {
    sqlx::query_as::<_, EventComment>(
        r#"
        INSERT INTO event_comments (
            event_id,
            author_user_id,
            body
        )
        VALUES ($1, $2, $3)
        RETURNING
            id,
            event_id,
            author_user_id,
            body,
            created_at,
            updated_at
        "#,
    )
    .bind(event_id)
    .bind(author_user_id)
    .bind(body)
    .fetch_one(&state.db)
    .await
    .map_err(EventError::Database)
}

async fn delete_event_comment(
    state: &AppState,
    event_id: Uuid,
    comment_id: Uuid,
    user_id: Uuid,
) -> Result<bool, EventError> {
    let result = sqlx::query(
        r#"
        DELETE FROM event_comments c
        USING events e
        WHERE c.id = $1
            AND c.event_id = $2
            AND e.id = c.event_id
            AND (
                c.author_user_id = $3
                OR e.organizer_user_id = $3
            )
        "#,
    )
    .bind(comment_id)
    .bind(event_id)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map_err(EventError::Database)?;

    Ok(result.rows_affected() > 0)
}

async fn fetch_event_activity(
    state: &AppState,
    event_id: Uuid,
) -> Result<Vec<EventActivityEntry>, EventError> {
    sqlx::query_as::<_, EventActivityEntry>(
        r#"
        SELECT
            id,
            event_id,
            actor_user_id,
            activity_type,
            subject_type,
            subject_id,
            message,
            created_at
        FROM event_activity_entries
        WHERE event_id = $1
        ORDER BY created_at DESC, id DESC
        "#,
    )
    .bind(event_id)
    .fetch_all(&state.db)
    .await
    .map_err(EventError::Database)
}

async fn insert_event_activity(
    state: &AppState,
    event_id: Uuid,
    actor_user_id: Option<Uuid>,
    activity_type: &str,
    subject_type: Option<&str>,
    subject_id: Option<Uuid>,
    message: impl Into<String>,
) -> Result<EventActivityEntry, EventError> {
    sqlx::query_as::<_, EventActivityEntry>(
        r#"
        INSERT INTO event_activity_entries (
            event_id,
            actor_user_id,
            activity_type,
            subject_type,
            subject_id,
            message
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id,
            event_id,
            actor_user_id,
            activity_type,
            subject_type,
            subject_id,
            message,
            created_at
        "#,
    )
    .bind(event_id)
    .bind(actor_user_id)
    .bind(activity_type)
    .bind(subject_type)
    .bind(subject_id)
    .bind(message.into())
    .fetch_one(&state.db)
    .await
    .map_err(EventError::Database)
}

async fn fetch_event_rsvp_invitations(
    state: &AppState,
    event_id: Uuid,
) -> Result<Vec<Invitation>, EventError> {
    sqlx::query_as::<_, Invitation>(
        r#"
        SELECT
            id,
            event_id,
            invitee_user_id,
            invitee_email,
            status,
            share_token,
            rsvp_status,
            rsvp_responded_at,
            created_at,
            updated_at
        FROM invitations
        WHERE event_id = $1
            AND status <> 'revoked'
            AND rsvp_status IN ('yes', 'no', 'maybe')
            AND rsvp_responded_at IS NOT NULL
        ORDER BY
            CASE rsvp_status
                WHEN 'yes' THEN 1
                WHEN 'maybe' THEN 2
                WHEN 'no' THEN 3
                ELSE 4
            END,
            rsvp_responded_at DESC,
            created_at DESC
        "#,
    )
    .bind(event_id)
    .fetch_all(&state.db)
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

fn rsvp_list_response(event_id: Uuid, invitations: Vec<Invitation>) -> EventRsvpListResponse {
    let mut response = EventRsvpListResponse {
        event_id,
        coming: Vec::new(),
        declined: Vec::new(),
        maybe: Vec::new(),
    };

    for invitation in invitations {
        let Some(entry) = rsvp_list_entry(&invitation) else {
            continue;
        };
        match entry.rsvp_status.as_str() {
            RSVP_STATUS_YES => response.coming.push(entry),
            RSVP_STATUS_NO => response.declined.push(entry),
            RSVP_STATUS_MAYBE => response.maybe.push(entry),
            _ => {}
        }
    }

    response
}

fn rsvp_list_entry(invitation: &Invitation) -> Option<RsvpListEntry> {
    let rsvp_status = invitation.rsvp_status.as_deref()?;
    let rsvp_responded_at = invitation.rsvp_responded_at?;
    if !matches!(
        rsvp_status,
        RSVP_STATUS_YES | RSVP_STATUS_NO | RSVP_STATUS_MAYBE
    ) {
        return None;
    }

    Some(RsvpListEntry {
        invitation_id: invitation.id,
        invitee_user_id: invitation.invitee_user_id,
        invitee_email: invitation.invitee_email.clone(),
        rsvp_status: rsvp_status.to_owned(),
        rsvp_responded_at,
    })
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

fn invitation_url(state: &AppState, invitation: &Invitation) -> Result<String, EventError> {
    let invitation_path = format!("/invitations/{}", invitation.share_token);
    state
        .auth_links
        .login_url(Some(&invitation_path))
        .map_err(EventError::BadRequest)
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
    if email.is_empty()
        || email.len() > 320
        || email.chars().any(char::is_whitespace)
        || !email.contains('@')
    {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "invitee email must be a valid address"
        )));
    }

    let Some((local, domain)) = email.split_once('@') else {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "invitee email must be a valid address"
        )));
    };
    if local.is_empty() || domain.is_empty() || !domain.contains('.') {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "invitee email must be a valid address"
        )));
    }

    Ok(email)
}

fn normalize_share_token(value: &str) -> Result<String, EventError> {
    let token = value.trim();
    if token.is_empty() {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "invitation token is required"
        )));
    }
    if token.len() > 128 {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "invitation token is invalid"
        )));
    }
    if !token
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || matches!(char, '-' | '_'))
    {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "invitation token is invalid"
        )));
    }

    Ok(token.to_owned())
}

fn normalize_rsvp_status(value: &str) -> Result<String, EventError> {
    let status = value.trim().to_ascii_lowercase();
    match status.as_str() {
        RSVP_STATUS_YES | RSVP_STATUS_NO | RSVP_STATUS_MAYBE => Ok(status),
        _ => Err(EventError::BadRequest(anyhow::anyhow!(
            "rsvp status must be yes, no, or maybe"
        ))),
    }
}

fn normalize_comment_body(value: &str) -> Result<String, EventError> {
    let body = value.trim();
    if body.is_empty() {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "comment body is required"
        )));
    }
    if body.chars().count() > MAX_COMMENT_BODY_CHARS {
        return Err(EventError::BadRequest(anyhow::anyhow!(
            "comment body must be {MAX_COMMENT_BODY_CHARS} characters or fewer"
        )));
    }

    Ok(body.to_owned())
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
    let starts_at = event.starts_at.to_rfc3339();
    let location = event
        .location_name
        .as_deref()
        .unwrap_or("Location to be announced");

    templates::event_invitation(EventInvitationTemplate {
        invitee_email,
        event_title: &event.title,
        starts_at: &starts_at,
        location,
        invitation_url,
    })
}

async fn send_rsvp_confirmation(
    state: &AppState,
    event: &Event,
    invitation: &Invitation,
    recipient: &str,
) -> bool {
    let Some(rsvp_status) = invitation.rsvp_status.as_deref() else {
        return false;
    };
    let message = rsvp_confirmation_message(event, recipient, rsvp_status);

    match state.email.send(message).await {
        Ok(EmailSendOutcome::Sent { message_id }) => {
            tracing::info!(
                %message_id,
                event_id = %event.id,
                invitation_id = %invitation.id,
                invitee_email = %recipient,
                rsvp_status = %rsvp_status,
                "rsvp confirmation email sent"
            );
            true
        }
        Ok(EmailSendOutcome::Skipped { reason }) => {
            tracing::warn!(
                %reason,
                event_id = %event.id,
                invitation_id = %invitation.id,
                invitee_email = %recipient,
                rsvp_status = %rsvp_status,
                "rsvp confirmation email skipped"
            );
            false
        }
        Err(error) => {
            tracing::warn!(
                %error,
                event_id = %event.id,
                invitation_id = %invitation.id,
                invitee_email = %recipient,
                rsvp_status = %rsvp_status,
                "rsvp confirmation email failed"
            );
            false
        }
    }
}

fn rsvp_confirmation_message(event: &Event, recipient: &str, rsvp_status: &str) -> EmailMessage {
    let starts_at = event.starts_at.to_rfc3339();
    let location = event
        .location_name
        .as_deref()
        .unwrap_or("Location to be announced");

    templates::rsvp_confirmation(RsvpConfirmationTemplate {
        recipient_email: recipient,
        event_title: &event.title,
        rsvp_status_label: rsvp_status_label(rsvp_status),
        starts_at: &starts_at,
        location,
    })
}

fn rsvp_status_label(status: &str) -> &'static str {
    match status {
        RSVP_STATUS_YES => "yes",
        RSVP_STATUS_NO => "no",
        RSVP_STATUS_MAYBE => "maybe",
        _ => "unknown",
    }
}

#[cfg(test)]
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
            Self::Forbidden(error) => (StatusCode::FORBIDDEN, "forbidden", error.to_string()),
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

        crate::error::json_error(status, code, message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cover_image_extension, escape_html, normalize_comment_body, normalize_email,
        normalize_event_input, normalize_rsvp_status, normalize_share_token,
        parse_required_datetime, rsvp_list_response, rsvp_status_label, validate_pdf_content_type,
        EventMultipartInput,
    };
    use crate::models::invitation::{Invitation, INVITATION_STATUS_ACCEPTED};
    use uuid::Uuid;

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
        assert!(normalize_email("friend @example.com").is_err());
        assert!(normalize_email("friend@example").is_err());
    }

    #[test]
    fn escapes_invitation_email_html() {
        assert_eq!(
            escape_html("<Gather & friends>"),
            "&lt;Gather &amp; friends&gt;"
        );
    }

    #[test]
    fn validates_share_tokens() {
        assert_eq!(
            normalize_share_token("  abc123  ").expect("token accepted"),
            "abc123"
        );
        assert!(normalize_share_token("").is_err());
        assert!(normalize_share_token(&"x".repeat(129)).is_err());
        assert!(normalize_share_token("../secret").is_err());
        assert!(normalize_share_token("token with spaces").is_err());
    }

    #[test]
    fn validates_rsvp_statuses() {
        assert_eq!(normalize_rsvp_status(" YES ").expect("yes accepted"), "yes");
        assert_eq!(
            normalize_rsvp_status("maybe").expect("maybe accepted"),
            "maybe"
        );
        assert!(normalize_rsvp_status("soon").is_err());
        assert_eq!(rsvp_status_label("no"), "no");
    }

    #[test]
    fn normalizes_comment_body() {
        assert_eq!(
            normalize_comment_body("  See you there.  ").expect("comment accepted"),
            "See you there."
        );
        assert!(normalize_comment_body("   ").is_err());
        assert!(normalize_comment_body(&"x".repeat(2000)).is_ok());
        assert!(normalize_comment_body(&"x".repeat(2001)).is_err());
    }

    #[test]
    fn normalizes_event_input_and_rejects_bad_times() {
        let input = EventMultipartInput {
            title: Some("  Birthday dinner  ".to_owned()),
            description: Some("  Bring dessert  ".to_owned()),
            starts_at: Some("2026-08-01T18:00:00Z".to_owned()),
            ends_at: Some("2026-08-01T20:00:00Z".to_owned()),
            timezone: Some(" America/New_York ".to_owned()),
            location_name: Some("  Main Hall  ".to_owned()),
            location_address: Some("  123 Example St  ".to_owned()),
            cover_image: None,
            pdf_attachments: Vec::new(),
        };

        let normalized = normalize_event_input(input).expect("event should normalize");

        assert_eq!(normalized.title, "Birthday dinner");
        assert_eq!(normalized.description.as_deref(), Some("Bring dessert"));
        assert_eq!(
            normalized.ends_at.expect("ends_at").to_rfc3339(),
            "2026-08-01T20:00:00+00:00"
        );
        assert_eq!(normalized.timezone.as_deref(), Some("America/New_York"));
        assert_eq!(normalized.location_name.as_deref(), Some("Main Hall"));
        assert_eq!(
            normalized.location_address.as_deref(),
            Some("123 Example St")
        );

        assert!(normalize_event_input(EventMultipartInput {
            title: Some("Birthday dinner".to_owned()),
            description: None,
            starts_at: Some("2026-08-01T18:00:00Z".to_owned()),
            ends_at: Some("2026-08-01T18:00:00Z".to_owned()),
            timezone: None,
            location_name: None,
            location_address: None,
            cover_image: None,
            pdf_attachments: Vec::new(),
        })
        .is_err());
    }

    #[test]
    fn groups_rsvp_list_by_status() {
        let event_id = Uuid::new_v4();
        let yes = invitation_with_rsvp(event_id, "yes@example.com", Some("yes"));
        let no = invitation_with_rsvp(event_id, "no@example.com", Some("no"));
        let maybe = invitation_with_rsvp(event_id, "maybe@example.com", Some("maybe"));
        let pending = invitation_with_rsvp(event_id, "pending@example.com", None);

        let response = rsvp_list_response(event_id, vec![yes, no, maybe, pending]);

        assert_eq!(response.event_id, event_id);
        assert_eq!(response.coming.len(), 1);
        assert_eq!(
            response.coming[0].invitee_email.as_deref(),
            Some("yes@example.com")
        );
        assert_eq!(response.declined.len(), 1);
        assert_eq!(
            response.declined[0].invitee_email.as_deref(),
            Some("no@example.com")
        );
        assert_eq!(response.maybe.len(), 1);
        assert_eq!(
            response.maybe[0].invitee_email.as_deref(),
            Some("maybe@example.com")
        );
    }

    #[test]
    fn rsvp_list_preserves_user_identity_and_ignores_invalid_entries() {
        let event_id = Uuid::new_v4();
        let invitee_user_id = Uuid::new_v4();
        let mut user_invitation = invitation_with_rsvp(event_id, "user@example.com", Some("yes"));
        user_invitation.invitee_user_id = Some(invitee_user_id);
        user_invitation.invitee_email = None;
        let unknown_status = invitation_with_rsvp(event_id, "unknown@example.com", Some("later"));

        let response = rsvp_list_response(event_id, vec![user_invitation, unknown_status]);

        assert_eq!(response.coming.len(), 1);
        assert_eq!(response.coming[0].invitee_user_id, Some(invitee_user_id));
        assert_eq!(response.coming[0].invitee_email, None);
        assert!(response.declined.is_empty());
        assert!(response.maybe.is_empty());
    }

    fn invitation_with_rsvp(
        event_id: Uuid,
        invitee_email: &str,
        rsvp_status: Option<&str>,
    ) -> Invitation {
        let now = parse_required_datetime(Some("2026-08-01T18:00:00Z".to_owned()), "now")
            .expect("test timestamp");

        Invitation {
            id: Uuid::new_v4(),
            event_id,
            invitee_user_id: None,
            invitee_email: Some(invitee_email.to_owned()),
            status: INVITATION_STATUS_ACCEPTED.to_owned(),
            share_token: Uuid::new_v4().to_string(),
            rsvp_status: rsvp_status.map(str::to_owned),
            rsvp_responded_at: rsvp_status.map(|_| now),
            created_at: now,
            updated_at: now,
        }
    }
}
