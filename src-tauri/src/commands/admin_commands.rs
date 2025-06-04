use crate::{
    config::{parse_backup_url, save_settings, AppSettings},
    error::Result as AppResult,
    models::User,
    state::AppState,
    utils, AppError,
};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

#[derive(Deserialize)]
pub struct LoginPayload {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    success: bool,
    message: String,
    username: Option<String>,
}

#[tauri::command]
pub async fn login_admin(
    payload: LoginPayload,
    state: State<'_, AppState>,
) -> AppResult<LoginResponse> {
    tracing::info!("Login attempt for user: {}", payload.username);

    let user = sqlx::query_as!(
        User,
        "SELECT id as `id!`, username, password_hash, created_at, updated_at FROM users WHERE username = ?",
        payload.username
    )
    .fetch_optional(&state.db_pool)
    .await?;

    match user {
        Some(u) => match utils::verify_password(&payload.password, &u.password_hash) {
            Ok(true) => {
                tracing::info!("User '{}' logged in successfully.", payload.username);
                Ok(LoginResponse {
                    success: true,
                    message: "Login successful".to_string(),
                    username: Some(u.username),
                })
            }
            Ok(false) => {
                tracing::warn!("Invalid password for user '{}'.", payload.username);
                Ok(LoginResponse {
                    success: false,
                    message: "Invalid username or password".to_string(),
                    username: None,
                })
            }
            Err(e) => {
                tracing::error!(
                    "Error verifying password for user '{}': {}",
                    payload.username,
                    e
                );
                Err(e)
            }
        },
        None => {
            tracing::warn!("User '{}' not found.", payload.username);
            Ok(LoginResponse {
                success: false,
                message: "Invalid username or password".to_string(),
                username: None,
            })
        }
    }
}

#[tauri::command]
pub async fn get_app_settings(app_state: tauri::State<'_, AppState>) -> AppResult<AppSettings> {
    Ok(app_state.settings.read().await.clone())
}

#[derive(Deserialize, Debug)]
pub struct UpdateAppSettingsPayload {
    pub language: Option<String>,
    pub theme: Option<String>,
    pub timezone: Option<String>,
    pub backup_url: Option<String>, // Might be Some("") to clear
    pub backup_period_hours: Option<u64>,
}

#[tauri::command]
pub async fn update_app_settings(
    app_handle: AppHandle,
    app_state: tauri::State<'_, AppState>,
    payload: UpdateAppSettingsPayload,
) -> AppResult<()> {
    let mut settings = app_state.settings.write().await;
    let mut changed = false;

    if let Some(lang) = payload.language {
        settings.language = lang;
        changed = true;
    }
    if let Some(theme) = payload.theme {
        settings.theme = theme;
        changed = true;
    }
    if let Some(tz) = payload.timezone {
        let gym_tz: Tz = tz.parse().map_err(|e| {
            tracing::error!("Failed to parse timezone from settings: {}", e);
            return AppError::Config("Invalid gym timezone configuration.".to_string());
        })?;
        settings.timezone = tz;
        changed = true;
    }
    if payload.backup_url.is_some() {
        let full_url = payload.backup_url.as_deref().unwrap_or("");
        let url = parse_backup_url(full_url);
        match url {
            Ok(_) => {
                settings.backup_url = Some(full_url.to_string());
                changed = true;
            }
            Err(e) => {
                tracing::error!("Invalid backup URL format: {}", e);
                return Err(AppError::Config("Invalid backup URL format".to_string()));
            }
        }
    }
    if payload.backup_period_hours.is_some() {
        settings.backup_period_hours = payload.backup_period_hours;
        changed = true;
    }

    if changed {
        save_settings(&app_handle, &settings).await?;
        app_handle
            .emit("settings_changed", settings.clone())
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to emit settings_changed event: {}", e);
            });
    }
    Ok(())
}
