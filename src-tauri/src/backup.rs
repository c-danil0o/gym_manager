use crate::config::parse_backup_url;
use crate::db::get_database_path;
use crate::error::{AppError, ErrorCodes, Result as AppResult, TranslatableError};
use crate::models::CronCheck;
use crate::AppState;
use base64::engine::general_purpose;
use base64::Engine;
use chrono::Local;
use reqwest::header::CONTENT_TYPE;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::pool::PoolConnection;
use sqlx::{ConnectOptions, Sqlite};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{Emitter, Manager};
use tokio::time::interval;

const BACKUP_CHECK_INTERVAL_MINUTES: u64 = 30;

async fn execute_vacuum(
    mut conn: PoolConnection<Sqlite>,
    backup_path: &str,
) -> Result<(), AppError> {
    let query_string = format!("VACUUM INTO '{}'", backup_path);

    sqlx::query(&query_string)
        .execute(&mut *conn)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute VACUUM command: {}", e);
            AppError::Sqlx(e)
        })?;

    Ok(())
}

async fn perform_backup(app_handle: &tauri::AppHandle) -> AppResult<()> {
    tracing::info!("Starting database backup process...");

    let app_state = app_handle.state::<AppState>();
    let backup_url = app_state.settings.read().await.backup_url.clone();

    if backup_url.is_none() {
        tracing::warn!("Backup URL is not configured, skipping backup.");
        return Ok(());
    }
    let url_data = parse_backup_url(backup_url.unwrap().as_str());
    if url_data.is_err() {
        tracing::error!("Invalid backup URL format: {}", url_data.unwrap_err());
        return Err(AppError::BackupFailed(
            "Invalid backup URL format".to_string(),
        ));
    }
    let (backup_url, token) = url_data.unwrap();

    let db_path = get_database_path(app_handle)?;
    let backup_path = db_path.join("backup.tmp.sqlite");
    let backup_path_str = backup_path.to_str().ok_or_else(|| {
        AppError::Config(format!(
            "Failed to convert backup path to string: {:?}",
            backup_path
        ))
    })?;

    tracing::info!("Creating temporary backup file at: {:?}", backup_path);

    let conn = app_state.db_pool.acquire().await.map_err(|e| {
        tracing::error!("Failed to acquire database connection: {}", e);
        AppError::Sqlx(e)
    })?;

    tracing::info!("Executing VACUUM INTO temporary file: {}", backup_path_str);
    execute_vacuum(conn, backup_path_str).await?;

    let db_file_bytes = tokio::fs::read(&backup_path).await.map_err(|e| {
        tracing::error!("Failed to read database file: {}", e);
        AppError::Io(e)
    })?;

    tokio::fs::remove_file(&backup_path).await.map_err(|e| {
        tracing::error!("Failed to remove temporary backup file: {}", e);
        AppError::Io(e)
    })?;

    let client = reqwest::Client::new();
    let upload_endpoint = format!("{}/backup", backup_url);

    let res = client
        .post(&upload_endpoint)
        .header(CONTENT_TYPE, "application/octet-stream")
        .header("X-Api-Key", token)
        .body(db_file_bytes)
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                let now = Local::now().naive_local();
                save_last_backup_date(&app_handle.state(), now, "success").await?;
                tracing::info!("Backup completed successfully at {}", now);
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "No error message".to_string());
                tracing::error!("Backup failed with status {}: {}", status, error_text);
                save_last_backup_date(&app_handle.state(), Local::now().naive_local(), "fail")
                    .await?;
                return Err(AppError::BackupFailed(format!(
                    "Backup failed with status {}: {}",
                    status, error_text
                )));
            }
        }
        Err(e) => {
            tracing::error!("Backup request failed: {:?}", e);
            return Err(AppError::BackupFailed(
                "Backup failed. Gateway error response!".to_string(),
            ));
        }
    }

    Ok(())
}

async fn load_last_backup_date(
    app_state: &tauri::State<'_, AppState>,
) -> AppResult<chrono::NaiveDateTime> {
    println!("✅ ✅✅✅✅backup loading from DB");
    let check = sqlx::query_as!(
          CronCheck,
          "SELECT id as `id!`, last_check_time, check_type, status, created_at, updated_at FROM cron_checks WHERE check_type = 'backup' AND status='success' ORDER BY last_check_time DESC LIMIT 1",
      )
      .fetch_optional(&app_state.db_pool)
      .await?;

    match check {
        Some(check) => Ok(check.last_check_time),
        None => Ok(chrono::NaiveDateTime::from_timestamp(0, 0)),
    }
}

async fn save_last_backup_date(
    app_state: &tauri::State<'_, AppState>,
    date: chrono::NaiveDateTime,
    status: &str,
) -> AppResult<()> {
    // craete if not exists
    sqlx::query!(
        "INSERT INTO cron_checks (last_check_time, check_type, status, created_at) VALUES (?, 'backup', ?, CURRENT_TIMESTAMP)
         ON CONFLICT(check_type) DO UPDATE SET last_check_time = excluded.last_check_time, status = excluded.status, updated_at = CURRENT_TIMESTAMP",
        date,
        status
    )
    .execute(&app_state.db_pool)
    .await?;

    // Try to get the lock without waiting
    if let Ok(mut last_backup) = app_state.last_backup.try_write() {
        *last_backup = Some(date);
    } else {
        tracing::warn!("Could not acquire write lock on last_backup, skipping in-memory update");
    }
    Ok(())
}
pub async fn is_backup_needed(app_state: &tauri::State<'_, AppState>) -> AppResult<bool> {
    tracing::info!("Checking if backup is needed...");
    let backup_enabled = app_state.settings.read().await.backup_enabled;
    if !backup_enabled {
        tracing::info!("Backup is disabled, skipping backup check.");
        return Ok(false);
    }
    let today = Local::now().naive_local();
    let last_backup_in_memory = *app_state.last_backup.read().await;

    let last_backup: chrono::NaiveDateTime;
    if let Some(in_memory_date) = last_backup_in_memory {
        last_backup = in_memory_date;
    } else {
        last_backup = load_last_backup_date(app_state).await?;
        if let Ok(mut last_backup_state) = app_state.last_backup.try_write() {
            *last_backup_state = Some(last_backup);
        }
    }
    let backup_period = app_state.settings.read().await.backup_period_hours;
    let backup_period = match backup_period {
        Some(period) => {
            println!(
                "Backup period is set to {} hours, skipping backup check.",
                period
            );
            period as i64
        }
        None => {
            return Ok(false);
        }
    };
    let backup_threshold = last_backup
        .checked_add_signed(chrono::Duration::hours(backup_period))
        .unwrap_or_else(|| chrono::NaiveDateTime::from_timestamp(0, 0));

    if backup_threshold < today {
        return Ok(true);
    } else {
        println!("Backup check skipped: Last backup was within the configured period.");
        return Ok(false);
    }
}

/// Spawns the background task for periodic change checks.
pub fn spawn_backup_check_task(app_handle: tauri::AppHandle) {
    tracing::info!(
        "Spawning periodic backup change check task (Interval: {} minutes)",
        BACKUP_CHECK_INTERVAL_MINUTES,
    );

    tokio::spawn(async move {
        let check_interval_duration = Duration::from_secs(BACKUP_CHECK_INTERVAL_MINUTES * 60);
        let mut check_timer = interval(check_interval_duration);
        check_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        // check_timer.tick().await;

        loop {
            check_timer.tick().await;
            let is_backup_needed = is_backup_needed(&app_handle.state()).await;
            match is_backup_needed {
                Ok(needed) => {
                    if needed {
                        let mut message = "performing_backup";
                        tracing::info!("Backup is needed, performing backup...");
                        if let Err(e) = perform_backup(&app_handle).await {
                            tracing::error!("Backup failed: {:?}", e);
                            message = "backup_failed";
                        }
                        let _ = &app_handle
                            .emit("status", message.to_string())
                            .unwrap_or_else(|e| {
                                tracing::warn!("Failed to emit status event: {}", e);
                            });
                    } else {
                        tracing::info!("No backup needed at this time.");
                    }
                }
                Err(e) => {
                    tracing::error!("Error checking if backup is needed: {:?}", e);
                }
            }
        }
    });
}

pub async fn manual_trigger_backup(app_handle: tauri::AppHandle) -> AppResult<()> {
    return perform_backup(&app_handle).await;
}
async fn restore_local_backup(backup_path: &std::path::PathBuf, db_path: &std::path::PathBuf) {
    if backup_path.exists() {
        tracing::info!(
            "Restoring local backup from {:?} to {:?}",
            backup_path,
            db_path
        );
        if let Err(e) = tokio::fs::rename(backup_path, db_path).await {
            tracing::error!("CRITICAL FAILURE: Could not restore local backup. The database file might be missing. Error: {}", e);
        }
    }
}

pub async fn check_db_integrity(db_path: &PathBuf) -> AppResult<()> {
    tracing::info!("Performing integrity check on: {:?}", db_path);

    if !db_path.exists() {
        tracing::info!("Database file does not exist, creating it...");
        return Err(AppError::Config(format!(
            "Database file does not exist at: {:?}",
            db_path
        )));
    }

    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(false)
        .read_only(true);

    let mut conn = options
        .connect()
        .await
        .map_err(|e| AppError::Config(format!("Failed to connect to database: {}", e)))?;

    // The PRAGMA returns a single row with the text 'ok' if the DB is not corrupt.
    let result: (String,) = sqlx::query_as("PRAGMA integrity_check;")
        .fetch_one(&mut conn)
        .await
        .map_err(|e| AppError::Sqlx(e))?;

    if result.0.to_lowercase() == "ok" {
        tracing::info!("Integrity check passed for: {:?}", db_path);
        Ok(())
    } else {
        tracing::error!(
            "Integrity check failed for {:?}. Result: {}",
            db_path,
            result.0
        );
        Err(AppError::RestoreFailed(result.0))
    }
}

async fn download_and_verify_backup(
    temp_path: &PathBuf,
    download_url: String,
    token: String,
) -> AppResult<()> {
    tracing::info!("Downloading backup file to: {:?}", temp_path);

    // Download the file
    let client = reqwest::Client::new();
    let res = client
        .get(&download_url)
        .header("X-Api-Key", token)
        .send()
        .await;

    let response = match res {
        Ok(response) => {
            if response.status().is_success() {
                tracing::info!("Backup file downloaded successfully.");
                response
            } else {
                let status = response.status();
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "No error message".to_string());
                tracing::error!(
                    "Backup download failed with status {}: {}",
                    status,
                    error_text
                );
                return Err(AppError::RestoreFailed(format!(
                    "Backup download failed with status {}: {}",
                    status, error_text
                )));
            }
        }
        Err(e) => {
            tracing::error!("Failed to download backup file: {:?}", e);
            return Err(AppError::Reqwest(e));
        }
    };

    // The body is base64 encoded by our Lambda
    let encoded_body = response.text().await;
    let body = match encoded_body {
        Ok(body) => body,
        Err(e) => {
            tracing::error!("Failed to read response body: {:?}", e);
            return Err(AppError::Reqwest(e));
        }
    };

    let db_bytes = general_purpose::STANDARD.decode(body).map_err(|e| {
        tracing::error!("Failed to decode base64 response body: {:?}", e);
        AppError::Base64Decode(e)
    })?;

    tokio::fs::write(temp_path, db_bytes).await.map_err(|e| {
        tracing::error!("Failed to write backup file: {:?}", e);
        AppError::Io(e)
    })?;
    tracing::info!("Backup file downloaded successfully.");
    check_db_integrity(temp_path).await?;
    Ok(())
}

#[tauri::command]
pub async fn restore_from_backup(
    app_handle: tauri::AppHandle,
    version_id: Option<String>,
) -> AppResult<String> {
    tracing::info!("Starting database restore process...");

    let app_state = app_handle.state::<AppState>();
    let backup_url = app_state.settings.read().await.backup_url.clone();

    if backup_url.is_none() {
        tracing::warn!("Backup URL is not configured, skipping backup.");
        return Err(AppError::Translatable(TranslatableError::new(
            ErrorCodes::BACKUP_URL_NOT_SET,
            "Backup URL is not set. Please configure it in the settings.",
        )));
    }
    let url_data = parse_backup_url(backup_url.unwrap().as_str());
    if url_data.is_err() {
        tracing::error!("Invalid backup URL format: {}", url_data.unwrap_err());
        return Err(AppError::Translatable(TranslatableError::new(
            ErrorCodes::INVALID_BACKUP_URL,
            "Invalid backup URL format",
        )));
    }
    let (backup_url, token) = url_data.unwrap();
    let mut download_url = format!("{}/backup", backup_url);
    if let Some(vid) = version_id {
        if !vid.is_empty() && vid != "null" {
            download_url = format!("{}?versionId={}", download_url, vid);
        }
    }
    println!("Download URL: {}", download_url);

    app_state.db_pool.close().await;
    tracing::info!("Database connection pool closed.");

    let db_folder = get_database_path(&app_handle)?;
    let temp_path = db_folder.join("gym_data_tmp.sqlite");
    let db_path = db_folder.join("gym_data.sqlite");
    let backup_path = db_folder.join("gym_data_backup.sqlite");
    let wal_path = db_folder.join("gym_data.sqlite-wal");
    let shm_path = db_folder.join("gym_data.sqlite-shm");

    tracing::info!("Backing up current database to: {:?}", backup_path);
    if let Err(e) = tokio::fs::rename(&db_path, &backup_path).await {
        let err_msg = format!(
            "Failed to create local backup of current DB: {}. Please restart.",
            e
        );
        tracing::error!("{}", err_msg);
        return Err(AppError::Io(e));
    }

    match download_and_verify_backup(&temp_path, download_url, token).await {
        Ok(_) => {
            tracing::info!(
                "Verification successful. Replacing current database with downloaded version."
            );
            if let Err(e) = tokio::fs::rename(&temp_path, &db_path).await {
                restore_local_backup(&backup_path, &db_path).await;
                return Err(AppError::RestoreFailed(format!("CRITICAL: Failed to replace database after verification. Local backup has been restored. Error: {}", e)));
            }
            if backup_path.exists() {
                if let Err(e) = tokio::fs::remove_file(&backup_path).await {
                    tracing::warn!(
                        "Could not remove temporary backup file {:?}: {}",
                        backup_path,
                        e
                    );
                }
            }
            tracing::info!("Database file successfully overwritten from remote backup.");
        }
        Err(e) => {
            tracing::error!("Restore failed: {}. Reverting to local backup.", e);
            restore_local_backup(&backup_path, &db_path).await;

            if temp_path.exists() {
                let _ = tokio::fs::remove_file(&temp_path).await;
            }

            return Err(AppError::RestoreFailed(format!("Restore failed due to an error: {}. Your previous data has been restored. Please restart the application to reconnect.", e)));
        }
    }

    tracing::info!("Deleting old WAL/SHM files if they exist...");
    let _ = tokio::fs::remove_file(&wal_path).await;
    let _ = tokio::fs::remove_file(&shm_path).await;
    Ok("Restore successful. Please restart the application.".to_string())
}
