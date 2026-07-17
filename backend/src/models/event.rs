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

#[cfg(test)]
mod tests {
    use super::{Event, EventSummary};
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    #[test]
    fn event_summary_keeps_dashboard_fields_only() {
        let starts_at = DateTime::parse_from_rfc3339("2026-08-01T18:00:00Z")
            .expect("timestamp")
            .with_timezone(&Utc);
        let created_at = DateTime::parse_from_rfc3339("2026-07-01T12:00:00Z")
            .expect("timestamp")
            .with_timezone(&Utc);
        let event = Event {
            id: Uuid::new_v4(),
            organizer_user_id: Uuid::new_v4(),
            title: "Dinner".to_owned(),
            description: Some("Details stay off the summary".to_owned()),
            starts_at,
            ends_at: None,
            timezone: Some("UTC".to_owned()),
            location_name: Some("Main Hall".to_owned()),
            location_address: Some("123 Example St".to_owned()),
            cover_image_object_key: Some("events/cover.jpg".to_owned()),
            pdf_attachment_object_keys: vec!["events/menu.pdf".to_owned()],
            created_at,
            updated_at: created_at,
        };

        let summary = EventSummary::from(event.clone());

        assert_eq!(summary.id, event.id);
        assert_eq!(summary.organizer_user_id, event.organizer_user_id);
        assert_eq!(summary.title, "Dinner");
        assert_eq!(summary.starts_at, starts_at);
        assert_eq!(summary.location_name.as_deref(), Some("Main Hall"));
        assert_eq!(
            summary.cover_image_object_key.as_deref(),
            Some("events/cover.jpg")
        );
    }
}
