use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tokio::fs; // Use tokio's async fs

const CONFIG_FILENAME: &str = "app_settings.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSettings {
    pub backup_url: Option<String>,
    pub backup_period_hours: Option<u64>,
    pub language: String,
    pub theme: String,
    pub timezone: String,
    pub backup_enabled: bool,
    pub gym_name: String,
    pub gym_code: String,
    pub sync_enabled: bool,
    pub supabase_url: Option<String>,
    pub supabase_key: Option<String>,
    pub sync_period_minutes: Option<u64>,
}
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            theme: "light".to_string(),
            timezone: "Europe/Belgrade".to_string(),
            backup_url: None,
            backup_period_hours: Some(12),
            backup_enabled: false,
            gym_name: "Gym".to_string(),
            gym_code: {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let code: String = (0..10)
                    .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
                    .collect();
                code
            },
            sync_enabled: false,
            supabase_url: None,
            supabase_key: None,
            sync_period_minutes: Some(5),
        }
    }
}

fn get_config_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|_| AppError::Config("Cannot determine app config directory".to_string()))?;
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }
    Ok(config_dir.join(CONFIG_FILENAME))
}

pub async fn load_settings(app_handle: &AppHandle) -> Result<AppSettings> {
    let config_path = get_config_path(app_handle)?;
    if !config_path.exists() {
        tracing::info!(
            "Settings file not found at {:?}, creating with defaults.",
            config_path
        );
        let default_settings = AppSettings::default();
        save_settings(app_handle, &default_settings).await?;
        return Ok(default_settings);
    }
    let content = fs::read_to_string(config_path).await?;

    // Try to parse settings normally first
    let settings = match serde_json::from_str::<AppSettings>(&content) {
        Ok(settings) => settings,
        Err(_) => {
            // If parsing fails, merge with defaults
            tracing::info!("Settings file has missing fields, merging with defaults");
            let default_settings = AppSettings::default();
            let default_json = serde_json::to_value(&default_settings)?;

            if let (Ok(mut default_obj), Ok(existing_obj)) = (
                serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(default_json),
                serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&content)
            ) {
                // Merge existing values into defaults
                for (key, value) in existing_obj {
                    default_obj.insert(key, value);
                }

                serde_json::from_value(serde_json::Value::Object(default_obj))
                    .unwrap_or(default_settings)
            } else {
                default_settings
            }
        }
    };

    // Save the merged settings back to ensure new fields are persisted
    save_settings(app_handle, &settings).await?;

    Ok(settings)
}

pub async fn save_settings(app_handle: &AppHandle, settings: &AppSettings) -> Result<()> {
    let config_path = get_config_path(app_handle)?;
    let content = serde_json::to_string_pretty(settings)?;
    fs::write(config_path, content).await?;
    Ok(())
}

pub fn parse_backup_url(full_url: &str) -> Result<(String, String)> {
    let parsed_url = url::Url::parse(full_url)
        .map_err(|_| AppError::Config("Invalid backup URL format".to_string()))?;

    let token = parsed_url
        .query_pairs()
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.into_owned())
        .ok_or_else(|| {
            AppError::Config("Backup URL must contain a 'token' query parameter".to_string())
        })?;

    let base_url = format!(
        "{}://{}/{}",
        parsed_url.scheme(),
        parsed_url
            .host_str()
            .ok_or(AppError::Config("URL missing host".to_string()))?,
        parsed_url.path().trim_start_matches('/')
    );

    Ok((base_url, token))
}
