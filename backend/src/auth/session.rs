#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::models::user::User;

const MCTAI_AUTH_PROVIDER: &str = "mctai";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MctaiSessionClaims {
    pub sub: String,
    pub email: String,
    #[serde(default)]
    pub email_verified: bool,
    pub name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedSession {
    pub user: User,
    pub provider: &'static str,
    pub subject: String,
}

pub async fn upsert_user_from_mctai_claims(
    pool: &PgPool,
    claims: &MctaiSessionClaims,
) -> Result<AuthenticatedSession, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (
            email,
            oauth_provider,
            oauth_subject,
            display_name,
            full_name,
            email_verified,
            email_verified_at,
            last_seen_at
        )
        VALUES (
            $1,
            $2,
            $3,
            $4,
            $4,
            $5,
            CASE WHEN $5 THEN NOW() ELSE NULL END,
            NOW()
        )
        ON CONFLICT (oauth_provider, oauth_subject)
            WHERE oauth_provider IS NOT NULL AND oauth_subject IS NOT NULL
        DO UPDATE SET
            email = EXCLUDED.email,
            display_name = COALESCE(EXCLUDED.display_name, users.display_name),
            full_name = COALESCE(EXCLUDED.full_name, users.full_name),
            email_verified = EXCLUDED.email_verified,
            email_verified_at = CASE
                WHEN EXCLUDED.email_verified THEN COALESCE(users.email_verified_at, NOW())
                ELSE NULL
            END,
            updated_at = NOW(),
            last_seen_at = NOW()
        RETURNING
            id,
            email,
            password_hash,
            oauth_provider,
            oauth_subject,
            display_name,
            full_name,
            avatar_object_key,
            email_verified,
            email_verified_at,
            created_at,
            updated_at,
            last_seen_at
        "#,
    )
    .bind(&claims.email)
    .bind(MCTAI_AUTH_PROVIDER)
    .bind(&claims.sub)
    .bind(claims.name.as_deref())
    .bind(claims.email_verified)
    .fetch_one(pool)
    .await?;

    Ok(AuthenticatedSession {
        user,
        provider: MCTAI_AUTH_PROVIDER,
        subject: claims.sub.clone(),
    })
}
