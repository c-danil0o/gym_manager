use sqlx::SqlitePool;

use crate::config::AppSettings;

pub struct AppState {
    pub db_pool: SqlitePool,
    pub settings: tokio::sync::RwLock<AppSettings>,
}

impl AppState {
    pub fn new(db_pool: SqlitePool, settings: AppSettings) -> Self {
        Self {
            db_pool,
            settings: tokio::sync::RwLock::new(settings),
        }
    }
}
