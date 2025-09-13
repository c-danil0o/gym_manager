use sqlx::SqlitePool;

use crate::config::AppSettings;
use crate::sync::SupabaseClient;

pub struct AppState {
    pub db_pool: SqlitePool,
    pub settings: tokio::sync::RwLock<AppSettings>,
    pub last_membership_check: tokio::sync::RwLock<Option<chrono::NaiveDateTime>>,
    pub last_backup: tokio::sync::RwLock<Option<chrono::NaiveDateTime>>,
    pub sync_mutex: tokio::sync::Mutex<()>,
    pub supabase_client: tokio::sync::RwLock<Option<SupabaseClient>>,
}

impl AppState {
    pub fn new(db_pool: SqlitePool, settings: AppSettings) -> Self {
        Self {
            db_pool,
            settings: tokio::sync::RwLock::new(settings),
            last_membership_check: tokio::sync::RwLock::new(None),
            last_backup: tokio::sync::RwLock::new(None),
            sync_mutex: tokio::sync::Mutex::new(()),
            supabase_client: tokio::sync::RwLock::new(None),
        }
    }
}

impl AppState {
    pub async fn get_supabase_client(&self) -> crate::error::Result<SupabaseClient> {
        let settings = self.settings.read().await;
        let url = settings
            .supabase_url
            .as_ref()
            .ok_or_else(|| crate::error::AppError::Sync("Supabase URL not configured".to_string()))?;
        let key = settings
            .supabase_key
            .as_ref()
            .ok_or_else(|| crate::error::AppError::Sync("Supabase key not configured".to_string()))?;

        // Check if we have a cached client
        {
            let client_guard = self.supabase_client.read().await;
            if let Some(client) = client_guard.as_ref() {
              return Ok(client.clone());
            }
        }
        // Create new client and cache it
        let client = SupabaseClient::new(url, key)?;

        {
            let mut client_guard = self.supabase_client.write().await;
            *client_guard = Some(client.clone());
        }

        Ok(client)
    }
}
