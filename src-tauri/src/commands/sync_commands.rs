use crate::error::Result as AppResult;
use crate::sync::{test_supabase_connection, perform_full_sync, sync_pending_changes, get_pending_changes_info, manual_trigger_sync};
use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub async fn test_supabase_connection_command(
    url: String,
    key: String,
) -> AppResult<()> {
    test_supabase_connection(&url, &key).await
}

#[tauri::command]
pub async fn perform_full_sync_command(
    app_handle: tauri::AppHandle,
) -> AppResult<()> {
    perform_full_sync(&app_handle).await
}

#[tauri::command]
pub async fn sync_pending_changes_command(
    app_handle: tauri::AppHandle,
) -> AppResult<()> {
    sync_pending_changes(&app_handle).await
}

#[tauri::command]
pub async fn manual_trigger_sync_command(
    app_handle: tauri::AppHandle,
) -> AppResult<()> {
    manual_trigger_sync(app_handle).await
}

#[tauri::command]
pub async fn get_pending_changes_info_command(
    app_handle: tauri::AppHandle,
) -> AppResult<serde_json::Value> {
    get_pending_changes_info(&app_handle).await
}

#[tauri::command]
pub async fn clear_all_pending_changes(
    state: State<'_, AppState>,
) -> AppResult<()> {
    let pool = &state.db_pool;
    
    sqlx::query("DELETE FROM pending_changes")
        .execute(pool)
        .await
        .map_err(|e| crate::error::AppError::Database(format!("Failed to clear pending changes: {}", e)))?;
    
    Ok(())
}

#[tauri::command]
pub async fn clear_failed_pending_changes(
    state: State<'_, AppState>,
) -> AppResult<()> {
    let pool = &state.db_pool;
    
    sqlx::query("DELETE FROM pending_changes WHERE retry_count >= 3")
        .execute(pool)
        .await
        .map_err(|e| crate::error::AppError::Database(format!("Failed to clear failed pending changes: {}", e)))?;
    
    Ok(())
}