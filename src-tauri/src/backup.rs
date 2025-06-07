use crate::config::parse_backup_url;
use crate::db::get_database_path;
use crate::error::{AppError, Result as AppResult};
use crate::models::CronCheck;
use crate::AppState;
use chrono::Local;
use reqwest::header::CONTENT_TYPE;
use std::time::Duration;
use tauri::Manager;
use tokio::time::interval;

const BACKUP_CHECK_INTERVAL_MINUTES: u64 = 30;

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
    let db_file_bytes = tokio::fs::read(&db_path).await.map_err(|e| {
        tracing::error!("Failed to read database file: {}", e);
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
    let today = Local::now().naive_local();
    let last_backup = app_state.last_backup.read().await;
    let last_backup_time = last_backup
        .clone()
        .unwrap_or(load_last_backup_date(&app_state).await?);
    drop(last_backup);
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
    let backup_threshold = last_backup_time
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
        let check_interval_duration = Duration::from_secs(BACKUP_CHECK_INTERVAL_MINUTES * 4);
        let mut check_timer = interval(check_interval_duration);
        check_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        // check_timer.tick().await;

        loop {
            check_timer.tick().await;
            let is_backup_needed = is_backup_needed(&app_handle.state()).await;
            match is_backup_needed {
                Ok(needed) => {
                    if needed {
                        tracing::info!("Backup is needed, performing backup...");
                        if let Err(e) = perform_backup(&app_handle).await {
                            tracing::error!("Backup failed: {:?}", e);
                        }
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
