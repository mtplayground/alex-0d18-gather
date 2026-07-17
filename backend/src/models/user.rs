#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
    pub oauth_provider: Option<String>,
    pub oauth_subject: Option<String>,
    pub display_name: Option<String>,
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    pub avatar_object_key: Option<String>,
    pub email_verified: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub email: String,
    pub password_hash: Option<String>,
    pub oauth_provider: Option<String>,
    pub oauth_subject: Option<String>,
    pub display_name: Option<String>,
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    pub avatar_object_key: Option<String>,
    pub email_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    pub avatar_object_key: Option<String>,
    pub email_verified: bool,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            full_name: user.full_name,
            bio: user.bio,
            location: user.location,
            website_url: user.website_url,
            avatar_object_key: user.avatar_object_key,
            email_verified: user.email_verified,
        }
    }
}
