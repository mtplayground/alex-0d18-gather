#![allow(dead_code)]

use std::time::Duration;

use anyhow::Context;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::{Builder as S3ConfigBuilder, Region},
    presigning::PresigningConfig,
    primitives::ByteStream,
    Client,
};
use bytes::Bytes;

use crate::config::ObjectStorageConfig;

#[derive(Debug, Clone)]
pub struct ObjectStorage {
    client: Client,
    bucket: String,
    prefix: String,
}

#[derive(Debug, Clone)]
pub struct StoredObject {
    pub relative_key: String,
    pub full_key: String,
    pub content_length: usize,
}

impl ObjectStorage {
    pub fn from_config(config: &ObjectStorageConfig) -> anyhow::Result<Self> {
        let credentials = Credentials::new(
            config.access_key_id.clone(),
            config.secret_access_key.clone(),
            None,
            None,
            "object-storage-env",
        );
        let s3_config = S3ConfigBuilder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .credentials_provider(credentials)
            .endpoint_url(config.endpoint.clone())
            .force_path_style(config.force_path_style)
            .build();

        Ok(Self {
            client: Client::from_conf(s3_config),
            bucket: config.bucket.clone(),
            prefix: config.prefix.clone(),
        })
    }

    pub fn full_key(&self, relative_key: &str) -> anyhow::Result<String> {
        validate_relative_key(relative_key)?;
        Ok(format!("{}{}", self.prefix, relative_key))
    }

    pub async fn put_object(
        &self,
        relative_key: &str,
        body: Bytes,
        content_type: Option<&str>,
    ) -> anyhow::Result<StoredObject> {
        let full_key = self.full_key(relative_key)?;
        let content_length = body.len();
        let mut request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .content_length(content_length as i64)
            .body(ByteStream::from(body));

        if let Some(content_type) = content_type {
            request = request.content_type(content_type);
        }

        request
            .send()
            .await
            .with_context(|| format!("failed to upload object {relative_key}"))?;

        Ok(StoredObject {
            relative_key: relative_key.to_owned(),
            full_key,
            content_length,
        })
    }

    pub async fn delete_object(&self, relative_key: &str) -> anyhow::Result<()> {
        let full_key = self.full_key(relative_key)?;
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await
            .with_context(|| format!("failed to delete object {relative_key}"))?;

        Ok(())
    }

    pub async fn presigned_get_url(
        &self,
        relative_key: &str,
        expires_in: Duration,
    ) -> anyhow::Result<String> {
        let full_key = self.full_key(relative_key)?;
        let presigning_config = PresigningConfig::expires_in(expires_in)
            .context("failed to build S3 presigning config")?;
        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .presigned(presigning_config)
            .await
            .with_context(|| format!("failed to presign object {relative_key}"))?;

        Ok(presigned.uri().to_string())
    }
}

fn validate_relative_key(relative_key: &str) -> anyhow::Result<()> {
    if relative_key.is_empty() {
        return Err(anyhow::anyhow!("object key must not be empty"));
    }
    if relative_key.starts_with('/') {
        return Err(anyhow::anyhow!("object key must be relative"));
    }
    if relative_key.split('/').any(|segment| segment == "..") {
        return Err(anyhow::anyhow!("object key must not contain .. segments"));
    }

    Ok(())
}
