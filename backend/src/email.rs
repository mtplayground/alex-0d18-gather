#![allow(dead_code)]

use std::fmt;

use anyhow::Context;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::config::EmailConfig;

pub mod templates;

#[derive(Clone)]
pub struct EmailClient {
    inner: Option<ConfiguredEmailClient>,
}

#[derive(Clone)]
struct ConfiguredEmailClient {
    http: reqwest::Client,
    url: String,
    app_token: String,
}

#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub to: Vec<String>,
    pub subject: String,
    pub html: Option<String>,
    pub text: Option<String>,
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailSendOutcome {
    Sent { message_id: String },
    Skipped { reason: &'static str },
}

#[derive(Debug, Serialize)]
struct EmailPayload<'a> {
    to: &'a [String],
    subject: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct EmailProxyResponse {
    id: String,
}

impl EmailClient {
    pub fn from_config(config: Option<&EmailConfig>) -> Self {
        let inner = config.map(|config| ConfiguredEmailClient {
            http: reqwest::Client::new(),
            url: config.url.clone(),
            app_token: config.app_token.clone(),
        });

        Self { inner }
    }

    pub fn is_configured(&self) -> bool {
        self.inner.is_some()
    }

    pub async fn send(&self, message: EmailMessage) -> anyhow::Result<EmailSendOutcome> {
        let Some(inner) = &self.inner else {
            return Ok(EmailSendOutcome::Skipped {
                reason: "email proxy is not configured",
            });
        };

        message.validate()?;
        let payload = EmailPayload {
            to: &message.to,
            subject: &message.subject,
            html: message.html.as_deref(),
            text: message.text.as_deref(),
            reply_to: message.reply_to.as_deref(),
        };

        let response = inner
            .http
            .post(&inner.url)
            .bearer_auth(&inner.app_token)
            .json(&payload)
            .send()
            .await
            .context("failed to call email proxy")?;
        let status = response.status();

        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(anyhow::anyhow!("email rate limited; try again shortly"));
        }
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|error| format!("failed to read error body: {error}"));
            return Err(anyhow::anyhow!("email proxy failed: {status} {body}"));
        }

        let body = response
            .json::<EmailProxyResponse>()
            .await
            .context("failed to parse email proxy response")?;

        Ok(EmailSendOutcome::Sent {
            message_id: body.id,
        })
    }
}

impl EmailMessage {
    fn validate(&self) -> anyhow::Result<()> {
        if self.to.is_empty() {
            return Err(anyhow::anyhow!("email must have at least one recipient"));
        }
        if self.to.iter().any(|recipient| recipient.trim().is_empty()) {
            return Err(anyhow::anyhow!("email recipients must not be empty"));
        }
        if self.subject.trim().is_empty() {
            return Err(anyhow::anyhow!("email subject must not be empty"));
        }
        let has_html = self
            .html
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());
        let has_text = self
            .text
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());
        if !has_html && !has_text {
            return Err(anyhow::anyhow!("email must include html or text content"));
        }

        Ok(())
    }
}

impl fmt::Debug for EmailClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EmailClient")
            .field("configured", &self.is_configured())
            .finish()
    }
}

impl fmt::Debug for ConfiguredEmailClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfiguredEmailClient")
            .field("url", &self.url)
            .field("app_token", &"<redacted>")
            .finish()
    }
}
