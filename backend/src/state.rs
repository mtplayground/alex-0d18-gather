use sqlx::PgPool;

use crate::auth::links::AuthLinkConfig;
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
}

impl AppState {
    pub fn new(
        db: PgPool,
        storage: ObjectStorage,
        email: EmailClient,
        auth_links: AuthLinkConfig,
    ) -> Self {
        Self {
            db,
            storage,
            email,
            auth_links,
        }
    }
}
