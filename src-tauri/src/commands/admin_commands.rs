use crate::{
    error::{AppError, Result as AppResult},
    models::User,
    state::AppState,
    utils,
};
use serde::{Deserialize, Serialize};
use tauri::State;

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
        Some(u) => {
            match utils::verify_password(&payload.password, &u.password_hash) {
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
                    tracing::error!("Error verifying password for user '{}': {}", payload.username, e);
                    Err(e)
                }
            }
        }
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
