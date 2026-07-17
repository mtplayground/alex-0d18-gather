use sqlx::PgPool;

use crate::email::EmailClient;
use crate::storage::ObjectStorage;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    #[allow(dead_code)]
    pub storage: ObjectStorage,
    #[allow(dead_code)]
    pub email: EmailClient,
}

impl AppState {
    pub fn new(db: PgPool, storage: ObjectStorage, email: EmailClient) -> Self {
        Self { db, storage, email }
    }
}
