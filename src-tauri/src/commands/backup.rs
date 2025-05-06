use crate::{
    config::{self, AppSettings},
    error::{AppError, Result},
    backup::{self, apply_restore_data, request_restore_from_api, check_and_backup_if_needed}, // Import backup functions
    state::AppState, // Import AppState to access DB pool
};
use tauri::{AppHandle, Manager, State}; // Import Manager to access AppHandle from State


#[tauri::command]
pub async fn set_backup_url(app_handle: AppHandle, url: Option<String>) -> Result<()> {
    // Validate URL format before saving (allow empty/None to disable)
    if let Some(ref url_str) = url {
        if url_str.is_empty() {
             // Treat empty string as disabling backup
             return set_backup_url(app_handle, None).await;
        }
        // Basic validation: contains token and parses
        config::parse_backup_url(url_str)?;
    }

    let mut settings = config::load_settings(&app_handle).await?;
    settings.backup_url = url;
    config::save_settings(&app_handle, &settings).await?;
    tracing::info!("Backup URL updated.");
    Ok(())
}

#[tauri::command]
pub async fn get_backup_settings(app_handle: AppHandle) -> Result<AppSettings> {
    config::load_settings(&app_handle).await
}

#[tauri::command]
pub async fn trigger_backup_check(app_handle: AppHandle, state: State<'_, AppState>) -> Result<()> {
    tracing::info!("Manual backup check triggered.");
    let settings = config::load_settings(&app_handle).await?;
    if settings.backup_url.is_none() {
        return Err(AppError::Config("Backup URL not configured.".to_string()));
    }
    // Run the check function manually
    check_and_backup_if_needed(&state.db_pool, &settings).await;
     // Note: check_and_backup_if_needed handles its own logging/status updates.
     // We don't get direct success/failure back here easily unless refactored.
    Ok(())
}


#[tauri::command]
pub async fn trigger_restore(app_handle: AppHandle, state: State<'_, AppState>) -> Result<()> {
     tracing::info!("Manual restore triggered.");
     let settings = config::load_settings(&app_handle).await?;

     let Some(backup_url_str) = settings.backup_url.as_ref() else {
        return Err(AppError::Config("Backup URL not configured.".to_string()));
    };

    let (endpoint_base_url, token) = config::parse_backup_url(backup_url_str)?;

    // --- Confirmation Dialog ---
    let confirm = app_handle.dialog().confirm(
            "Restore from Backup?",
            "This will ERASE local Member, Membership, and Type data and replace it with the latest backup. This cannot be undone. Are you sure?",
        )
        .blocking_show(); // Use blocking_show if in async command context

     if !confirm {
         tracing::info!("Restore cancelled by user.");
         return Ok(()); // Not an error, just cancelled
     }


    // --- Perform Restore ---
    tracing::info!("User confirmed restore. Fetching data from API...");
    let backup_payload = request_restore_from_api(&endpoint_base_url, &token).await?;

    tracing::info!("Applying restore data...");
    apply_restore_data(&state.db_pool, backup_payload).await?;

     // Optionally trigger UI refresh after restore if needed
     app_handle.emit_all("restore_complete", ()).unwrap_or_else(|e| {
        tracing::warn!("Failed to emit restore_complete event: {}", e);
     });

    Ok(())
}


#[tauri::command]
pub async fn get_last_backup_status(state: State<'_, AppState>) -> Result<Option<serde_json::Value>> {
     // Fetch the latest status record from the new backup_status table
    let last_status_row = sqlx::query!(
        "SELECT status, last_check_time, last_successful_upload_time, error_message, created_at FROM backup_status ORDER BY created_at DESC LIMIT 1"
    )
    .fetch_optional(&state.db_pool)
    .await?;

     // Convert the row into a JSON object to send to the frontend
     match last_status_row {
        Some(row) => {
            let json_status = serde_json::json!({
                "status": row.status,
                "last_check_time": row.last_check_time,
                "last_successful_upload_time": row.last_successful_upload_time,
                "error_message": row.error_message,
                "log_time": row.created_at // when this status was logged
            });
            Ok(Some(json_status))
        }
        None => Ok(None), // No status logged yet
    }
}

// Remember to add these to commands/mod.rs and the main.rs handler list!
