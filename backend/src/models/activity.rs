#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub const ACTIVITY_EVENT_CREATED: &str = "event_created";
pub const ACTIVITY_RSVP_UPDATED: &str = "rsvp_updated";
pub const ACTIVITY_COMMENT_CREATED: &str = "comment_created";

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventActivityEntry {
    pub id: Uuid,
    pub event_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub activity_type: String,
    pub subject_type: Option<String>,
    pub subject_id: Option<Uuid>,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEventActivityEntry {
    pub event_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub activity_type: String,
    pub subject_type: Option<String>,
    pub subject_id: Option<Uuid>,
    pub message: String,
}
