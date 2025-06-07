use crate::{
    backup::manual_trigger_backup,
    config::{parse_backup_url, save_settings, AppSettings},
    dto::{BackupMetadata, UserDisplay, UserPayload},
    error::{ErrorCodes, Result as AppResult, TranslatableError},
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
    role: Option<String>,
}

#[tauri::command]
pub async fn login(payload: LoginPayload, state: State<'_, AppState>) -> AppResult<LoginResponse> {
    tracing::info!("Login attempt for user: {}", payload.username);

    let user = sqlx::query_as!(
        User,
        "SELECT id as `id!`, username, role, password_hash, created_at, updated_at FROM users WHERE username = ?",
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
                    role: Some(u.role),
                })
            }
            Ok(false) => {
                tracing::warn!("Invalid password for user '{}'.", payload.username);
                Ok(LoginResponse {
                    success: false,
                    message: "Invalid username or password".to_string(),
                    username: None,
                    role: None,
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
                role: None,
            })
        }
    }
}

#[tauri::command]
pub async fn get_all_users(app_state: tauri::State<'_, AppState>) -> AppResult<Vec<UserDisplay>> {
    let users = sqlx::query_as!(
        UserDisplay,
        "SELECT id as `id!`, username, role, created_at, updated_at FROM users"
    )
    .fetch_all(&app_state.db_pool)
    .await?;

    Ok(users)
}

#[tauri::command]
pub async fn get_user_by_id(
    app_state: tauri::State<'_, AppState>,
    user_id: i64,
) -> AppResult<Option<UserDisplay>> {
    let user = sqlx::query_as!(
        UserDisplay,
        "SELECT id as `id!`, username, role, created_at, updated_at FROM users WHERE id = ?",
        user_id
    )
    .fetch_optional(&app_state.db_pool)
    .await?;

    Ok(user)
}

#[tauri::command]
pub async fn save_user(
    app_state: tauri::State<'_, AppState>,
    payload: UserPayload,
) -> AppResult<UserDisplay> {
    if payload.username.is_empty() {
        return Err(AppError::Validation("Username cannot be empty".to_string()));
    }
    if payload.role != "admin" && payload.role != "user" {
        return Err(AppError::Validation(
            "Role must be 'admin' or 'user'".to_string(),
        ));
    }
    match payload.id {
        Some(id) => {
            // check if username already exists
            let existing_user = sqlx::query_as!(
                UserDisplay,
                "SELECT id as `id!`, username, role, created_at, updated_at FROM users WHERE username = ? AND id != ?",
                payload.username,
                id
            )
            .fetch_optional(&app_state.db_pool)
            .await?;
            if existing_user.is_some() {
                return Err(AppError::Translatable(TranslatableError::new(
                    ErrorCodes::USERNAME_ALREADY_EXISTS,
                    "Username already exists",
                )));
            }
            // update existing user
            let query = sqlx::query!(
                "UPDATE users SET username = ?, role = ? WHERE id = ?",
                payload.username,
                payload.role,
                id
            )
            .execute(&app_state.db_pool)
            .await?;
            if query.rows_affected() == 0 {
                return Err(AppError::NotFound("User not found".to_string()));
            }
            let updated_user = get_user_by_id(app_state, id).await?;

            if let Some(user) = updated_user {
                Ok(user)
            } else {
                Err(AppError::NotFound(
                    "User not found after update".to_string(),
                ))
            }
        }
        None => {
            // Create new user
            if payload.password.is_none() || payload.password.as_ref().unwrap().is_empty() {
                return Err(AppError::Validation(
                    "Password is required for new users".to_string(),
                ));
            }

            let hashed_password = utils::hash_password(&payload.password.unwrap_or_default())?;
            let user = sqlx::query_as!(
                UserDisplay,
                "INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?) RETURNING id as `id!`, username, role, created_at, updated_at",
                payload.username,
                hashed_password,
                payload.role
            )
            .fetch_one(&app_state.db_pool)
            .await?;
            Ok(user)
        }
    }
}

#[tauri::command]
pub async fn change_user_password(
    app_state: tauri::State<'_, AppState>,
    user_id: i64,
    new_password: String,
) -> AppResult<()> {
    if new_password.is_empty() {
        return Err(AppError::Validation(
            "New password cannot be empty".to_string(),
        ));
    }

    // Check if user exists
    let user_exists = sqlx::query!("SELECT COUNT(*) as count FROM users WHERE id = ?", user_id)
        .fetch_one(&app_state.db_pool)
        .await?
        .count
        > 0;

    if !user_exists {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    // Hash the new password
    let hashed_password = utils::hash_password(&new_password)?;

    // Update the user's password
    sqlx::query!(
        "UPDATE users SET password_hash = ? WHERE id = ?",
        hashed_password,
        user_id
    )
    .execute(&app_state.db_pool)
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn delete_user(app_state: tauri::State<'_, AppState>, user_id: i64) -> AppResult<()> {
    // Check if user exists
    let user_exists = sqlx::query!("SELECT COUNT(*) as count FROM users WHERE id = ?", user_id)
        .fetch_one(&app_state.db_pool)
        .await?
        .count
        > 0;

    if !user_exists {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    // Delete the user
    sqlx::query!("DELETE FROM users WHERE id = ?", user_id)
        .execute(&app_state.db_pool)
        .await?;

    Ok(())
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
        let _gym_tz: Tz = tz.parse().map_err(|e| {
            tracing::error!("Failed to parse timezone from settings: {}", e);
            return AppError::Translatable(TranslatableError::new(
                ErrorCodes::INVALID_TIMEZONE,
                "Invalid timezone configuration!",
            ));
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
                return Err(AppError::Translatable(TranslatableError::new(
                    ErrorCodes::INVALID_BACKUP_URL,
                    "Invalid backup URL format!",
                )));
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

#[tauri::command]
pub async fn get_remote_backup_metadata(
    app_state: tauri::State<'_, AppState>,
) -> AppResult<BackupMetadata> {
    let backup_url = app_state.settings.read().await.backup_url.clone();

    if backup_url.is_none() {
        tracing::warn!("Backup URL is not configured!.");
        return Err(AppError::Translatable(TranslatableError::new(
            ErrorCodes::BACKUP_URL_NOT_SET,
            "Backup URL is not configured!",
        )));
    }
    let url_data = parse_backup_url(backup_url.unwrap().as_str());
    if url_data.is_err() {
        tracing::error!("Invalid backup URL format: {}", url_data.unwrap_err());
        return Err(AppError::BackupFailed(
            "Invalid backup URL format".to_string(),
        ));
    }
    let (base_url, token) = url_data.unwrap();
    let metadata_url = format!("{}/backup/metadata", base_url);

    let client = reqwest::Client::new();
    let res = client
        .get(&metadata_url)
        .header("X-Api-Key", token) // <-- Add this header
        .send()
        .await;

    let response = match res {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("Failed to send request to backup metadata endpoint: {}", e);
            return Err(AppError::BackupFailed(
                "Failed to get metadata!".to_string(),
            ));
        }
    };

    if response.status().is_success() {
        response.json::<BackupMetadata>().await.map_err(|e| {
            tracing::error!("Failed to parse backup metadata response: {}", e);
            AppError::BackupFailed("Failed to parse metadata response".to_string())
        })
    } else {
        Err(AppError::BackupFailed(format!(
            "Failed to get metadata! Status: {}",
            response.status()
        )))
    }
}

#[tauri::command]
pub async fn trigger_backup(app_handle: tauri::AppHandle) -> AppResult<()> {
    tracing::info!("Triggering manual backup!");
    let result = manual_trigger_backup(app_handle).await;
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::info!("Manual backup failed");
            return Err(AppError::BackupFailed(format!(
                "Manual backup failed with error: {:?}",
                e
            )));
        }
    }
}
