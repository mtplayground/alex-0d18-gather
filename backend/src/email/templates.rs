use super::EmailMessage;

const BRAND_NAME: &str = "Gather";

#[derive(Debug, Clone, Copy)]
pub struct EventInvitationTemplate<'a> {
    pub invitee_email: &'a str,
    pub event_title: &'a str,
    pub starts_at: &'a str,
    pub location: &'a str,
    pub invitation_url: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct RsvpConfirmationTemplate<'a> {
    pub recipient_email: &'a str,
    pub event_title: &'a str,
    pub rsvp_status_label: &'a str,
    pub starts_at: &'a str,
    pub location: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct EventReminderTemplate<'a> {
    pub recipient_email: &'a str,
    pub event_title: &'a str,
    pub starts_at: &'a str,
    pub location: &'a str,
    pub event_url: &'a str,
    pub rsvp_status_label: &'a str,
}

pub fn verification(email: &str, auth_url: &str) -> EmailMessage {
    let escaped_url = escape_html(auth_url);
    let preheader = "Verify your email address and finish creating your Gather account.";
    let html = email_layout(
        "Complete your Gather registration",
        preheader,
        &format!(
            r#"<p>Welcome to {BRAND_NAME}.</p>
<p>Use this secure myClawTeam auth link to verify your email address and finish registration.</p>
{}"#,
            action_link(&escaped_url, "Complete registration")
        ),
    );
    let text = format!(
        "Welcome to {BRAND_NAME}.\n\nUse this secure myClawTeam auth link to verify your email address and finish registration:\n{auth_url}\n\nIf you did not request this email, you can ignore it."
    );

    EmailMessage {
        to: vec![email.to_owned()],
        subject: "Complete your Gather registration".to_owned(),
        html: Some(html),
        text: Some(text),
        reply_to: None,
    }
}

pub fn password_reset(email: &str, auth_url: &str) -> EmailMessage {
    let escaped_url = escape_html(auth_url);
    let preheader = "Continue account recovery with a secure myClawTeam auth link.";
    let html = email_layout(
        "Restore access to your Gather account",
        preheader,
        &format!(
            r#"<p>We received a request to restore access to your {BRAND_NAME} account.</p>
<p>Use this secure myClawTeam auth link to continue account recovery.</p>
{}"#,
            action_link(&escaped_url, "Continue account recovery")
        ),
    );
    let text = format!(
        "We received a request to restore access to your {BRAND_NAME} account.\n\nUse this secure myClawTeam auth link to continue account recovery:\n{auth_url}\n\nIf you did not request this email, you can ignore it."
    );

    EmailMessage {
        to: vec![email.to_owned()],
        subject: "Restore access to your Gather account".to_owned(),
        html: Some(html),
        text: Some(text),
        reply_to: None,
    }
}

pub fn event_invitation(template: EventInvitationTemplate<'_>) -> EmailMessage {
    let escaped_email = escape_html(template.invitee_email);
    let escaped_title = escape_html(template.event_title);
    let escaped_location = escape_html(template.location);
    let escaped_url = escape_html(template.invitation_url);
    let preheader = format!("You have been invited to {}.", template.event_title);
    let html = email_layout(
        "You are invited",
        &preheader,
        &format!(
            r#"<p>Hello {escaped_email},</p>
<p>You have been invited to <strong>{escaped_title}</strong>.</p>
<table role="presentation" cellpadding="0" cellspacing="0" style="margin:20px 0;width:100%;border-collapse:collapse;">
<tr><td style="padding:8px 0;color:#475569;width:84px;">When</td><td style="padding:8px 0;color:#0f172a;font-weight:600;">{}</td></tr>
<tr><td style="padding:8px 0;color:#475569;width:84px;">Where</td><td style="padding:8px 0;color:#0f172a;font-weight:600;">{escaped_location}</td></tr>
</table>
{}"#,
            escape_html(template.starts_at),
            action_link(&escaped_url, "View invitation")
        ),
    );
    let text = format!(
        "Hello {},\n\nYou have been invited to {}.\n\nWhen: {}\nWhere: {}\n\nView your invitation: {}",
        template.invitee_email,
        template.event_title,
        template.starts_at,
        template.location,
        template.invitation_url
    );

    EmailMessage {
        to: vec![template.invitee_email.to_owned()],
        subject: format!("Invitation: {}", template.event_title),
        html: Some(html),
        text: Some(text),
        reply_to: None,
    }
}

pub fn rsvp_confirmation(template: RsvpConfirmationTemplate<'_>) -> EmailMessage {
    let escaped_recipient = escape_html(template.recipient_email);
    let escaped_title = escape_html(template.event_title);
    let escaped_status = escape_html(template.rsvp_status_label);
    let escaped_location = escape_html(template.location);
    let preheader = format!(
        "Your RSVP for {} is confirmed as {}.",
        template.event_title, template.rsvp_status_label
    );
    let html = email_layout(
        "RSVP confirmed",
        &preheader,
        &format!(
            r#"<p>Hello {escaped_recipient},</p>
<p>Your RSVP for <strong>{escaped_title}</strong> is confirmed as <strong>{escaped_status}</strong>.</p>
<table role="presentation" cellpadding="0" cellspacing="0" style="margin:20px 0;width:100%;border-collapse:collapse;">
<tr><td style="padding:8px 0;color:#475569;width:84px;">When</td><td style="padding:8px 0;color:#0f172a;font-weight:600;">{}</td></tr>
<tr><td style="padding:8px 0;color:#475569;width:84px;">Where</td><td style="padding:8px 0;color:#0f172a;font-weight:600;">{escaped_location}</td></tr>
</table>"#,
            escape_html(template.starts_at)
        ),
    );
    let text = format!(
        "Hello {},\n\nYour RSVP for {} is confirmed as {}.\n\nWhen: {}\nWhere: {}",
        template.recipient_email,
        template.event_title,
        template.rsvp_status_label,
        template.starts_at,
        template.location
    );

    EmailMessage {
        to: vec![template.recipient_email.to_owned()],
        subject: format!("RSVP confirmed: {}", template.event_title),
        html: Some(html),
        text: Some(text),
        reply_to: None,
    }
}

pub fn event_reminder(template: EventReminderTemplate<'_>) -> EmailMessage {
    let escaped_title = escape_html(template.event_title);
    let escaped_location = escape_html(template.location);
    let escaped_status = escape_html(template.rsvp_status_label);
    let escaped_url = escape_html(template.event_url);
    let preheader = format!("Reminder: {} is coming up soon.", template.event_title);
    let html = email_layout(
        "Event reminder",
        &preheader,
        &format!(
            r#"<p><strong>{escaped_title}</strong> is coming up soon.</p>
<p>Your RSVP is currently marked as <strong>{escaped_status}</strong>.</p>
<table role="presentation" cellpadding="0" cellspacing="0" style="margin:20px 0;width:100%;border-collapse:collapse;">
<tr><td style="padding:8px 0;color:#475569;width:84px;">When</td><td style="padding:8px 0;color:#0f172a;font-weight:600;">{}</td></tr>
<tr><td style="padding:8px 0;color:#475569;width:84px;">Where</td><td style="padding:8px 0;color:#0f172a;font-weight:600;">{escaped_location}</td></tr>
</table>
{}"#,
            escape_html(template.starts_at),
            action_link(&escaped_url, "Open event")
        ),
    );
    let text = format!(
        "{} is coming up soon.\n\nYour RSVP is currently marked as {}.\n\nWhen: {}\nWhere: {}\n\nOpen event: {}",
        template.event_title,
        template.rsvp_status_label,
        template.starts_at,
        template.location,
        template.event_url
    );

    EmailMessage {
        to: vec![template.recipient_email.to_owned()],
        subject: format!("Reminder: {}", template.event_title),
        html: Some(html),
        text: Some(text),
        reply_to: None,
    }
}

fn email_layout(title: &str, preheader: &str, body: &str) -> String {
    format!(
        r#"<!doctype html>
<html>
<body style="margin:0;background:#f8fafc;color:#0f172a;font-family:Arial,sans-serif;">
<span style="display:none!important;opacity:0;color:transparent;height:0;width:0;overflow:hidden;">{}</span>
<table role="presentation" cellpadding="0" cellspacing="0" style="width:100%;border-collapse:collapse;background:#f8fafc;">
<tr>
<td style="padding:32px 16px;">
<table role="presentation" cellpadding="0" cellspacing="0" style="margin:0 auto;max-width:560px;width:100%;border-collapse:collapse;background:#ffffff;border:1px solid #e2e8f0;border-radius:8px;">
<tr>
<td style="padding:28px;">
<p style="margin:0 0 12px;color:#047857;font-size:14px;font-weight:700;">{BRAND_NAME}</p>
<h1 style="margin:0 0 18px;color:#0f172a;font-size:24px;line-height:1.25;">{}</h1>
<div style="color:#334155;font-size:16px;line-height:1.6;">{body}</div>
<p style="margin:24px 0 0;color:#64748b;font-size:13px;line-height:1.5;">If you did not request this email, you can ignore it.</p>
</td>
</tr>
</table>
</td>
</tr>
</table>
</body>
</html>"#,
        escape_html(preheader),
        escape_html(title)
    )
}

fn action_link(url: &str, label: &str) -> String {
    format!(
        r#"<p style="margin:24px 0;"><a href="{url}" style="display:inline-block;background:#047857;color:#ffffff;text-decoration:none;border-radius:6px;padding:12px 18px;font-weight:700;">{}</a></p>
<p style="margin:0;color:#64748b;font-size:13px;line-height:1.5;word-break:break-all;">{url}</p>"#,
        escape_html(label)
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::{
        event_invitation, event_reminder, password_reset, rsvp_confirmation, verification,
        EventInvitationTemplate, EventReminderTemplate, RsvpConfirmationTemplate,
    };

    #[test]
    fn verification_template_includes_both_formats() {
        let message = verification("person@example.com", "https://example.com/verify");

        assert_eq!(message.subject, "Complete your Gather registration");
        assert!(message
            .html
            .expect("html")
            .contains("Complete registration"));
        assert!(message
            .text
            .expect("text")
            .contains("https://example.com/verify"));
    }

    #[test]
    fn password_reset_template_includes_recovery_link() {
        let message = password_reset("person@example.com", "https://example.com/reset");

        assert_eq!(message.to, vec!["person@example.com"]);
        assert!(message
            .html
            .expect("html")
            .contains("Continue account recovery"));
        assert!(message
            .text
            .expect("text")
            .contains("https://example.com/reset"));
    }

    #[test]
    fn invitation_template_escapes_html() {
        let message = event_invitation(EventInvitationTemplate {
            invitee_email: "person@example.com",
            event_title: "<Launch>",
            starts_at: "2026-08-01T18:00:00Z",
            location: "HQ & Remote",
            invitation_url: "https://example.com/invite",
        });
        let html = message.html.expect("html");

        assert!(html.contains("&lt;Launch&gt;"));
        assert!(html.contains("HQ &amp; Remote"));
        assert!(message.text.expect("text").contains("<Launch>"));
    }

    #[test]
    fn rsvp_confirmation_template_names_status() {
        let message = rsvp_confirmation(RsvpConfirmationTemplate {
            recipient_email: "person@example.com",
            event_title: "Planning",
            rsvp_status_label: "maybe",
            starts_at: "2026-08-01T18:00:00Z",
            location: "Room 1",
        });

        assert_eq!(message.subject, "RSVP confirmed: Planning");
        assert!(message.html.expect("html").contains("maybe"));
        assert!(message.text.expect("text").contains("confirmed as maybe"));
    }

    #[test]
    fn reminder_template_includes_event_link() {
        let message = event_reminder(EventReminderTemplate {
            recipient_email: "person@example.com",
            event_title: "Planning",
            starts_at: "2026-08-01T18:00:00Z",
            location: "Room 1",
            event_url: "https://example.com/events/event-id",
            rsvp_status_label: "yes",
        });

        assert_eq!(message.subject, "Reminder: Planning");
        assert!(message.html.expect("html").contains("Open event"));
        assert!(message
            .text
            .expect("text")
            .contains("https://example.com/events/event-id"));
    }
}
