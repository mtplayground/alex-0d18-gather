use std::time::Duration;

use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    email::{
        templates::{self, EventReminderTemplate},
        EmailSendOutcome,
    },
    models::invitation::{RSVP_STATUS_MAYBE, RSVP_STATUS_YES},
    state::AppState,
};

const REMINDER_KIND_24H: &str = "event_24h";
const REMINDER_LOOKAHEAD: &str = "24 hours";
const REMINDER_BATCH_LIMIT: i64 = 200;
const REMINDER_POLL_INTERVAL: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, Clone, FromRow)]
struct ReminderCandidate {
    event_id: Uuid,
    invitation_id: Uuid,
    recipient_email: String,
    event_title: String,
    starts_at: DateTime<Utc>,
    location: String,
    rsvp_status: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ReminderRunSummary {
    pub candidates: usize,
    pub sent: usize,
    pub skipped: usize,
    pub failed: usize,
}

pub fn spawn_event_reminder_scheduler(state: AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(REMINDER_POLL_INTERVAL);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            interval.tick().await;

            match send_due_event_reminders(&state).await {
                Ok(summary) => {
                    if summary.candidates > 0 || summary.sent > 0 || summary.failed > 0 {
                        tracing::info!(
                            candidates = summary.candidates,
                            sent = summary.sent,
                            skipped = summary.skipped,
                            failed = summary.failed,
                            "event reminder scheduler run finished"
                        );
                    }
                }
                Err(error) => {
                    tracing::error!(%error, "event reminder scheduler run failed");
                }
            }
        }
    });
}

pub async fn send_due_event_reminders(state: &AppState) -> anyhow::Result<ReminderRunSummary> {
    let candidates = fetch_due_reminder_candidates(state).await?;
    let mut summary = ReminderRunSummary {
        candidates: candidates.len(),
        ..ReminderRunSummary::default()
    };

    for candidate in candidates {
        let Some(reminder_id) = reserve_reminder(state, &candidate).await? else {
            continue;
        };

        match send_reminder(state, &candidate).await {
            Ok(EmailSendOutcome::Sent { message_id }) => {
                summary.sent += 1;
                tracing::info!(
                    %message_id,
                    event_id = %candidate.event_id,
                    invitation_id = %candidate.invitation_id,
                    recipient_email = %candidate.recipient_email,
                    "event reminder email sent"
                );
            }
            Ok(EmailSendOutcome::Skipped { reason }) => {
                summary.skipped += 1;
                release_reminder_reservation(state, reminder_id).await?;
                tracing::warn!(
                    %reason,
                    event_id = %candidate.event_id,
                    invitation_id = %candidate.invitation_id,
                    recipient_email = %candidate.recipient_email,
                    "event reminder email skipped"
                );
            }
            Err(error) => {
                summary.failed += 1;
                release_reminder_reservation(state, reminder_id).await?;
                tracing::warn!(
                    %error,
                    event_id = %candidate.event_id,
                    invitation_id = %candidate.invitation_id,
                    recipient_email = %candidate.recipient_email,
                    "event reminder email failed"
                );
            }
        }
    }

    Ok(summary)
}

async fn fetch_due_reminder_candidates(state: &AppState) -> anyhow::Result<Vec<ReminderCandidate>> {
    sqlx::query_as::<_, ReminderCandidate>(
        r#"
        SELECT
            e.id AS event_id,
            i.id AS invitation_id,
            i.invitee_email AS recipient_email,
            e.title AS event_title,
            e.starts_at,
            COALESCE(e.location_name, e.location_address, 'Location to be announced') AS location,
            i.rsvp_status AS rsvp_status
        FROM events e
        JOIN invitations i ON i.event_id = e.id
        WHERE e.starts_at > NOW()
            AND e.starts_at <= NOW() + ($1::text)::interval
            AND i.invitee_email IS NOT NULL
            AND i.rsvp_status IN ($2, $3)
            AND i.status <> 'revoked'
            AND NOT EXISTS (
                SELECT 1
                FROM event_reminders r
                WHERE r.invitation_id = i.id
                    AND r.reminder_kind = $4
            )
        ORDER BY e.starts_at ASC, i.created_at ASC
        LIMIT $5
        "#,
    )
    .bind(REMINDER_LOOKAHEAD)
    .bind(RSVP_STATUS_YES)
    .bind(RSVP_STATUS_MAYBE)
    .bind(REMINDER_KIND_24H)
    .bind(REMINDER_BATCH_LIMIT)
    .fetch_all(&state.db)
    .await
    .context("failed to fetch due event reminder candidates")
}

async fn reserve_reminder(
    state: &AppState,
    candidate: &ReminderCandidate,
) -> anyhow::Result<Option<Uuid>> {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO event_reminders (
            event_id,
            invitation_id,
            reminder_kind
        )
        VALUES ($1, $2, $3)
        ON CONFLICT (invitation_id, reminder_kind) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(candidate.event_id)
    .bind(candidate.invitation_id)
    .bind(REMINDER_KIND_24H)
    .fetch_optional(&state.db)
    .await
    .context("failed to reserve event reminder")
}

async fn release_reminder_reservation(state: &AppState, reminder_id: Uuid) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        DELETE FROM event_reminders
        WHERE id = $1
        "#,
    )
    .bind(reminder_id)
    .execute(&state.db)
    .await
    .context("failed to release event reminder reservation")?;

    Ok(())
}

async fn send_reminder(
    state: &AppState,
    candidate: &ReminderCandidate,
) -> anyhow::Result<EmailSendOutcome> {
    let event_path = format!("/events/{}", candidate.event_id);
    let event_url = state
        .auth_links
        .login_url(Some(&event_path))
        .context("failed to build event reminder link")?;
    let starts_at = candidate.starts_at.to_rfc3339();
    let message = templates::event_reminder(EventReminderTemplate {
        recipient_email: &candidate.recipient_email,
        event_title: &candidate.event_title,
        starts_at: &starts_at,
        location: &candidate.location,
        event_url: &event_url,
        rsvp_status_label: rsvp_status_label(&candidate.rsvp_status),
    });

    state.email.send(message).await
}

fn rsvp_status_label(status: &str) -> &'static str {
    match status {
        RSVP_STATUS_YES => "yes",
        RSVP_STATUS_MAYBE => "maybe",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::{rsvp_status_label, ReminderRunSummary};

    #[test]
    fn labels_reminder_rsvp_statuses() {
        assert_eq!(rsvp_status_label("yes"), "yes");
        assert_eq!(rsvp_status_label("maybe"), "maybe");
        assert_eq!(rsvp_status_label("no"), "unknown");
    }

    #[test]
    fn reminder_summary_defaults_to_zero() {
        assert_eq!(ReminderRunSummary::default().sent, 0);
        assert_eq!(ReminderRunSummary::default().failed, 0);
    }
}
