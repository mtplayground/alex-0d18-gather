#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: Uuid,
    pub organizer_user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub timezone: Option<String>,
    pub location_name: Option<String>,
    pub location_address: Option<String>,
    pub cover_image_object_key: Option<String>,
    pub pdf_attachment_object_keys: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEvent {
    pub organizer_user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub timezone: Option<String>,
    pub location_name: Option<String>,
    pub location_address: Option<String>,
    pub cover_image_object_key: Option<String>,
    #[serde(default)]
    pub pdf_attachment_object_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    pub id: Uuid,
    pub organizer_user_id: Uuid,
    pub title: String,
    pub starts_at: DateTime<Utc>,
    pub location_name: Option<String>,
    pub cover_image_object_key: Option<String>,
}

impl From<Event> for EventSummary {
    fn from(event: Event) -> Self {
        Self {
            id: event.id,
            organizer_user_id: event.organizer_user_id,
            title: event.title,
            starts_at: event.starts_at,
            location_name: event.location_name,
            cover_image_object_key: event.cover_image_object_key,
        }
    }
}
