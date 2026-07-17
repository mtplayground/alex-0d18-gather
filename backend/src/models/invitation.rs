#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub const INVITATION_STATUS_PENDING: &str = "pending";
pub const INVITATION_STATUS_ACCEPTED: &str = "accepted";
pub const INVITATION_STATUS_DECLINED: &str = "declined";
pub const INVITATION_STATUS_REVOKED: &str = "revoked";

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invitation {
    pub id: Uuid,
    pub event_id: Uuid,
    pub invitee_user_id: Option<Uuid>,
    pub invitee_email: Option<String>,
    pub status: String,
    pub share_token: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewInvitation {
    pub event_id: Uuid,
    pub invitee_user_id: Option<Uuid>,
    pub invitee_email: Option<String>,
    #[serde(default = "default_invitation_status")]
    pub status: String,
    pub share_token: Option<String>,
}

pub fn default_invitation_status() -> String {
    INVITATION_STATUS_PENDING.to_owned()
}

pub fn is_valid_invitation_status(status: &str) -> bool {
    matches!(
        status,
        INVITATION_STATUS_PENDING
            | INVITATION_STATUS_ACCEPTED
            | INVITATION_STATUS_DECLINED
            | INVITATION_STATUS_REVOKED
    )
}
