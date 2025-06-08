use sqlx::SqlitePool;

use crate::config::AppSettings;

#[derive(Debug)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub settings: tokio::sync::RwLock<AppSettings>,
    pub last_membership_check: tokio::sync::RwLock<Option<chrono::NaiveDateTime>>,
    pub last_backup: tokio::sync::RwLock<Option<chrono::NaiveDateTime>>,
}

impl AppState {
    pub fn new(
        db_pool: SqlitePool,
        settings: AppSettings,
    ) -> Self {
        Self {
            db_pool,
            settings: tokio::sync::RwLock::new(settings),
            last_membership_check: tokio::sync::RwLock::new(None),
            last_backup: tokio::sync::RwLock::new(None),
        }
    }
}
