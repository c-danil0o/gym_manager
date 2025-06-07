use crate::config::AppSettings;
use crate::error::{AppError, Result};
use crate::models::{Member, Membership, MembershipType};
use chrono::NaiveDateTime;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::time::Duration;
use tokio::time::interval;

const CHANGE_CHECK_INTERVAL_HOURS: u64 = 2;

// Data structure to send/receive from Lambda
#[derive(Serialize, Deserialize, Debug, Clone)]
struct BackupPayload {
    members: Vec<Member>,
    memberships: Vec<Membership>,
    membership_types: Vec<MembershipType>,
    // users: Vec<User>,
}

#[derive(sqlx::FromRow, Debug)]
struct BackupStatus {
    id: i64,
    last_successful_upload_time: Option<NaiveDateTime>,
}

/// Checks if data in monitored tables has changed since the last successful backup.
// async fn needs_backup(pool: &SqlitePool) -> Result<Option<NaiveDateTime>> {
//     let last_success = sqlx::query_as!(
//         BackupStatus,
//         "SELECT id, last_successful_upload_time FROM backup_status WHERE status = 'upload_success' ORDER BY created_at DESC LIMIT 1"
//     )
//     .fetch_optional(pool)
//     .await?;

//     let last_success_time = last_success.and_then(|s| s.last_successful_upload_time);
//     tracing::info!("Last successful backup time: {:?}", last_success_time);

//     let max_updated_at = sqlx::query_scalar!(
//         r#"
//         SELECT MAX(max_updated) as "max_updated_at: Option<NaiveDateTime>"
//         FROM (
//             SELECT MAX(updated_at) as max_updated FROM members WHERE is_deleted = FALSE
//             UNION ALL
//             SELECT MAX(updated_at) as max_updated FROM memberships WHERE is_deleted = FALSE
//             UNION ALL
//             SELECT MAX(updated_at) as max_updated FROM membership_types WHERE is_deleted = FALSE
//         )
//         WHERE max_updated IS NOT NULL
//         "#
//     )
//     .fetch_one(pool) // Keep fetch_one as it returns Result<T, Error> which is often desired
//     .await?;

//     tracing::info!("Most recent data modification time: {:?}", max_updated_at);

//     match (max_updated_at, Option::from(last_success_time)) {
//         (Some(current_max_update), Some(last_backup)) => {
//             if current_max_update > last_backup {
//                 tracing::info!("Changes detected since last backup.");
//                 Ok(current_max_update) // Changes detected, return the timestamp of the data to be backed up
//             } else {
//                 tracing::info!("No changes detected since last backup.");
//                 Ok(None) // No changes
//             }
//         }
//         (Some(current_max_update), None) => {
//              tracing::info!("No previous successful backup found. Backup needed.");
//             Ok(current_max_update) // First backup needed
//         }
//         (None, _) => {
//             tracing::info!("No data found in monitored tables. No backup needed.");
//             Ok(None) // No data to back up
//         }
//     }
// }

/// Gathers data from the specified tables.
async fn gather_backup_data(pool: &SqlitePool) -> Result<BackupPayload> {
    // Update query_as to handle type conversions for i64 fields
    let members = sqlx::query_as!(Member, r#"SELECT
        id as "id!",
        card_id,
        short_card_id,
        first_name,
        last_name,
        email,
        phone,
        date_of_birth,
        created_at,
        updated_at,
        is_deleted as "is_deleted!"
    FROM members WHERE is_deleted = FALSE"#)
        .fetch_all(pool).await?;

    let memberships = sqlx::query_as!(Membership, r#"SELECT
        id as "id!",
        member_id as "member_id!",
        membership_type_id as "membership_type_id!",
        start_date,
        end_date,
        remaining_visits,
        status,
        purchase_date,
        created_at,
        updated_at,
        is_deleted as "is_deleted!"
    FROM memberships WHERE is_deleted = FALSE"#)
        .fetch_all(pool).await?;

    let membership_types = sqlx::query_as!(MembershipType, r#"SELECT
        id as "id!",
        name,
        duration_days,
        visit_limit,
        enter_by,
        price,
        description,
        created_at,
        updated_at,
        is_deleted as "is_deleted!"
    FROM membership_types WHERE is_deleted = FALSE"#)
        .fetch_all(pool).await?;

    Ok(BackupPayload {
        members,
        memberships,
        membership_types,
    })
}

/// Sends the backup data to the configured API endpoint.
async fn send_backup_to_api(
    endpoint_base_url: &str,
    token: &str,
    payload: &BackupPayload,
) -> Result<()> {
    let client = Client::new();
    let url = format!("{}?token={}", endpoint_base_url, token); // Append token for backup POST

    tracing::info!("Sending backup data to API: {}", endpoint_base_url);

    let response = client
        .post(url)
        .json(payload)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| AppError::BackupFailed(format!("HTTP request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        return Err(AppError::BackupFailed(format!(
            "Backup API returned error {}: {}",
            status, error_body
        )));
    }

    tracing::info!("Backup data successfully sent to API.");
    Ok(())
}

/// Records the status of the backup check/attempt.
// async fn log_backup_status(
//     pool: &SqlitePool,
//     status: &str, // 'checked_no_changes', 'upload_success', 'upload_failed'
//     upload_time: Option<NaiveDateTime>, // The timestamp of the data successfully uploaded
//     error_message: Option<String>,
// ) -> Result<()> {
//     let check_time = chrono::Utc::now().naive_utc();
//     sqlx::query!(
//         r#"
//         INSERT INTO backup_status (last_check_time, last_successful_upload_time, status, error_message)
//         VALUES (?, ?, ?, ?)
//         "#,
//         check_time,
//         upload_time,
//         status,
//         error_message
//     )
//     .execute(pool)
//     .await?;
//     Ok(())
// }

/// The main function for the periodic backup check task.
pub async fn check_and_backup_if_needed(pool: &SqlitePool, settings: &AppSettings) {
    let Some(backup_url_str) = settings.backup_url.as_ref() else {
        tracing::debug!("Backup URL not configured. Skipping check.");
        return;
    };

    let (endpoint_base_url, token) = match crate::config::parse_backup_url(backup_url_str) {
        Ok(parsed) => parsed,
        Err(e) => {
            tracing::error!("Invalid backup URL configuration: {}. Cannot perform backup check.", e);
            return;
        }
    };

    tracing::info!("Checking for data changes to backup...");

    match needs_backup(pool).await {
        Ok(Some(data_timestamp)) => {
            tracing::info!("Changes detected. Gathering data for backup...");
            match gather_backup_data(pool).await {
                Ok(payload) => {
                    match send_backup_to_api(&endpoint_base_url, &token, &payload).await {
                        Ok(_) => {
                            tracing::info!("Backup upload successful.");
                            if let Err(e) = log_backup_status(pool, "upload_success", Some(data_timestamp), None).await {
                                tracing::error!("Failed to log successful backup status: {}", e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Backup upload failed: {}", e);
                             if let Err(log_e) = log_backup_status(pool, "upload_failed", None, Some(e.to_string())).await {
                                tracing::error!("Failed to log failed backup status: {}", log_e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to gather data for backup: {}", e);
                     if let Err(log_e) = log_backup_status(pool, "upload_failed", None, Some(e.to_string())).await {
                        tracing::error!("Failed to log failed backup status after gather error: {}", log_e);
                    }
                }
            }
        }
        Ok(None) => {
            tracing::info!("No changes detected. No backup needed.");
             if let Err(e) = log_backup_status(pool, "checked_no_changes", None, None).await {
                 tracing::error!("Failed to log 'no changes' status: {}", e);
             }
        }
        Err(e) => {
            tracing::error!("Failed to check if backup is needed: {}", e);
            // Optionally log this failure specifically
            // let _ = log_backup_status(pool, "check_failed", None, Some(e.to_string())).await;
        }
    }
}

/// Fetches the backup data from the restore API endpoint.
pub async fn request_restore_from_api(
    endpoint_base_url: &str,
    token: &str,
) -> Result<BackupPayload> {
    let client = Client::new();
    let url = endpoint_base_url;

    tracing::info!("Requesting restore data from API: {}", url);

    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| AppError::RestoreFailed(format!("HTTP request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error body".to_string());
        return Err(AppError::RestoreFailed(format!(
            "Restore API returned error {}: {}",
            status, error_body
        )));
    }

    let payload: BackupPayload = response
        .json()
        .await
        .map_err(|e| AppError::RestoreFailed(format!("Failed to parse restore data: {}", e)))?;

    tracing::info!(
        "Successfully received restore data ({} members, {} memberships, {} types).",
        payload.members.len(), payload.memberships.len(), payload.membership_types.len()
    );
    Ok(payload)
}

/// Applies the restored data, deleting old data first within a transaction.
pub async fn apply_restore_data(pool: &SqlitePool, payload: BackupPayload) -> Result<()> {
    tracing::warn!("Applying restore data. Existing local data in restored tables will be DELETED.");

    let mut tx = pool.begin().await?; // Start transaction

    // Delete existing data (order matters for foreign keys if CASCADE isn't used everywhere)
    sqlx::query!("DELETE FROM memberships").execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM members").execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM membership_types").execute(&mut *tx).await?;

    // Insert restored data
    for member in payload.members {
        // Need to handle potential nulls correctly if columns allow them
        sqlx::query!(
            r#"
            INSERT INTO members (id, card_id, short_card_id, first_name, last_name, email, phone, date_of_birth, created_at, updated_at, is_deleted)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            member.id, // Insert explicit ID from backup
            member.card_id,
            member.short_card_id,
            member.first_name,
            member.last_name,
            member.email,
            member.phone,
            member.date_of_birth,
            member.created_at,
            member.updated_at,
            member.is_deleted
        ).execute(&mut *tx).await?;
    }

    for m_ship in payload.memberships {
         sqlx::query!(
            r#"
            INSERT INTO memberships (id, member_id, membership_type_id, start_date, end_date, remaining_visits, status, purchase_date, created_at, updated_at, is_deleted)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
             m_ship.id,
             m_ship.member_id,
             m_ship.membership_type_id,
             m_ship.start_date,
             m_ship.end_date,
             m_ship.remaining_visits,
             m_ship.status,
             m_ship.purchase_date,
             m_ship.created_at,
             m_ship.updated_at,
             m_ship.is_deleted
        ).execute(&mut *tx).await?;
    }

     for m_type in payload.membership_types {
         sqlx::query!(
            r#"
            INSERT INTO membership_types (id, name, duration_days, visit_limit, enter_by, price, description, created_at, updated_at, is_deleted)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
             m_type.id,
             m_type.name,
             m_type.duration_days,
             m_type.visit_limit,
             m_type.enter_by,
             m_type.price,
             m_type.description,
             m_type.created_at,
             m_type.updated_at,
             m_type.is_deleted
        ).execute(&mut *tx).await?;
    }


    tx.commit().await?; // Commit transaction

    tracing::info!("Successfully applied restore data.");
    Ok(())
}


/// Spawns the background task for periodic change checks.
pub fn spawn_backup_check_task(pool: SqlitePool, app_handle: tauri::AppHandle) {
    tracing::info!(
        "Spawning periodic backup change check task (Interval: {} hours)",
        CHANGE_CHECK_INTERVAL_HOURS,
    );

    tokio::spawn(async move {
        let check_interval_duration = Duration::from_secs(CHANGE_CHECK_INTERVAL_HOURS * 60 * 60);
        let mut check_timer = interval(check_interval_duration);

        loop {
            check_timer.tick().await;
            // Need copies for the async block
            let pool_clone = pool.clone();
            let handle_clone = app_handle.clone();

            // Load current settings inside the loop to pick up changes
            match crate::config::load_settings(&handle_clone).await {
                 Ok(settings) => {
                     if settings.backup_url.is_some() {
                         check_and_backup_if_needed(&pool_clone, &settings).await;
                     } else {
                         tracing::debug!("Periodic check skipped: Backup URL not configured.");
                     }
                 }
                 Err(e) => {
                    tracing::error!("Periodic check failed: Could not load settings: {}", e);
                 }
            }
        }
    });
}
