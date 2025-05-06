use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tokio::fs; // Use tokio's async fs

const CONFIG_FILENAME: &str = "app_settings.json";

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AppSettings {
    pub backup_url: Option<String>,
    // Add other settings here if needed
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
        tracing::info!("Settings file not found, returning default settings.");
        return Ok(AppSettings::default()); // Return default if file doesn't exist
    }

    let content = fs::read_to_string(config_path).await?;
    let settings: AppSettings =
        serde_json::from_str(&content).map_err(|e| AppError::Config(format!("Failed to parse settings file: {}", e)))?;

    Ok(settings)
}

pub async fn save_settings(app_handle: &AppHandle, settings: &AppSettings) -> Result<()> {
    let config_path = get_config_path(app_handle)?;
    let content = serde_json::to_string_pretty(settings)?;
    fs::write(config_path, content).await?;
    Ok(())
}

// Helper to parse URL and token (can be part of BackupConfig or called separately)
pub fn parse_backup_url(full_url: &str) -> Result<(String, String)> {
    let parsed_url = url::Url::parse(full_url)
        .map_err(|_| AppError::Config("Invalid backup URL format".to_string()))?;

    let token = parsed_url
        .query_pairs()
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.into_owned())
        .ok_or_else(|| AppError::Config("Backup URL must contain a 'token' query parameter".to_string()))?;

    // Reconstruct the base URL without the query string
    let base_url = format!(
        "{}://{}/{}",
        parsed_url.scheme(),
        parsed_url.host_str().ok_or(AppError::Config("URL missing host".to_string()))?,
        parsed_url.path().trim_start_matches('/') // Remove leading slash if present
    );


    Ok((base_url, token))
}
