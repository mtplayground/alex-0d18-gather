use sqlx::PgPool;

use crate::storage::ObjectStorage;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    #[allow(dead_code)]
    pub storage: ObjectStorage,
}

impl AppState {
    pub fn new(db: PgPool, storage: ObjectStorage) -> Self {
        Self { db, storage }
    }
}
