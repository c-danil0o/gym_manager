use sqlx::SqlitePool;

pub struct AppState {
    pub db_pool: SqlitePool,
    // Potentially add AppSettings here if loaded once at startup
    // pub settings: tokio::sync::RwLock<AppSettings>, // If state needs frequent async access/mutation
}

impl AppState {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }
}
