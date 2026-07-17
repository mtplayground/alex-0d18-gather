use sqlx::PgPool;

use crate::auth::{links::AuthLinkConfig, middleware::AuthVerifier};
use crate::email::EmailClient;
use crate::storage::ObjectStorage;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    #[allow(dead_code)]
    pub storage: ObjectStorage,
    #[allow(dead_code)]
    pub email: EmailClient,
    pub auth_links: AuthLinkConfig,
    pub auth: AuthVerifier,
}

impl AppState {
    pub fn new(
        db: PgPool,
        storage: ObjectStorage,
        email: EmailClient,
        auth_links: AuthLinkConfig,
        auth: AuthVerifier,
    ) -> Self {
        Self {
            db,
            storage,
            email,
            auth_links,
            auth,
        }
    }
}
