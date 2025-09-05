use crate::error::{AppError, Result as AppResult};
use crate::models::PendingChange;
use crate::state::AppState;
use chrono::NaiveDateTime;
use postgrest::Postgrest;
use serde_json::Value;
use sqlx::{Column, Row, TypeInfo};
use tauri::{Emitter, Manager};
use tokio::time::{interval, sleep, Duration, MissedTickBehavior};

const SYNC_CHECK_INTERVAL_SECONDS: u64 = 5;
const MAX_RETRY_ATTEMPTS: i64 = 3;
const BATCH_SIZE: i64 = 50;

pub struct SupabaseClient {
    client: Postgrest,
}

impl SupabaseClient {
    pub fn new(url: &str, key: &str) -> AppResult<Self> {
        // Ensure we're using the correct REST API endpoint
        let rest_url = if url.ends_with("/rest/v1") {
            url.to_string()
        } else if url.ends_with("/") {
            format!("{}rest/v1", url)
        } else {
            format!("{}/rest/v1", url)
        };

        tracing::info!("Creating Supabase client with URL: {}", rest_url);

        let client = Postgrest::new(&rest_url)
            .insert_header("apikey", key)
            .insert_header("Authorization", &format!("Bearer {}", key))
            .insert_header("Content-Type", "application/json")
            .insert_header("Prefer", "return=minimal");

        Ok(SupabaseClient { client })
    }

    async fn test_connection(&self) -> AppResult<()> {
        // Test connection by attempting to query a non-existent table
        // This should return 404 if the API is working but table doesn't exist
        // or 200 if table exists, both indicate working connection
        let response = self
            .client
            .from("connection_test_table_that_should_not_exist")
            .select("*")
            .limit(1)
            .execute()
            .await
            .map_err(|e| AppError::Sync(format!("Connection test failed: {}", e)))?;

        let status = response.status();
        tracing::info!("Connection test response status: {}", status);

        // Accept success, 404 (table not found), 406 (not acceptable) as valid responses
        // These indicate the API endpoint is working and credentials are valid
        if status.is_success() || status.as_u16() == 404 || status.as_u16() == 406 {
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(AppError::Sync(format!(
                "Connection test failed: {} - {}",
                status, text
            )))
        }
    }

    async fn ensure_tables_exist(&self) -> AppResult<()> {
        // Test if we can access the members table specifically
        let response = self
            .client
            .from("members")
            .select("id")
            .limit(1)
            .execute()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    tracing::info!("Tables are accessible via REST API");
                    Ok(())
                } else if status.as_u16() == 404 {
                    Err(AppError::Sync(
                        "Tables not found. Please ensure tables are created in Supabase and RLS policies allow access for service role.".to_string()
                    ))
                } else {
                    let text = resp.text().await.unwrap_or_default();
                    Err(AppError::Sync(format!(
                        "Table access check failed: {} - {}",
                        status, text
                    )))
                }
            }
            Err(e) => Err(AppError::Sync(format!(
                "Failed to check table access: {}",
                e
            ))),
        }
    }

    async fn clear_and_populate_table(&self, table_name: &str, data: &[Value]) -> AppResult<()> {
        // First, try to delete all records - use a condition that matches all records
        let delete_response = self
            .client
            .from(table_name)
            .delete()
            .gte("id", "0") // This should match all records since IDs are typically >= 0
            .execute()
            .await;

        match delete_response {
            Ok(response) => {
                if !response.status().is_success() {
                    let status = response.status();
                    if status.as_u16() == 404 {
                        return Err(AppError::Sync(format!(
                            "Table '{}' not found. Please ensure the table exists in Supabase and RLS policies allow access. Check SUPABASE_404_TROUBLESHOOTING.md for help.",
                            table_name
                        )));
                    } else {
                        let text = response.text().await.unwrap_or_default();
                        tracing::warn!(
                            "Failed to clear table {} - status: {} - {}",
                            table_name,
                            status,
                            text
                        );
                    }
                }
            }
            Err(e) => {
                return Err(AppError::Sync(format!(
                    "Failed to access table '{}': {}. Check your Supabase URL format and network connection.",
                    table_name, e
                )));
            }
        }

        // Then insert new data in batches
        if !data.is_empty() {
            for chunk in data.chunks(100) {
                let response = self
                    .client
                    .from(table_name)
                    .insert(
                        serde_json::to_string(&serde_json::Value::Array(chunk.to_vec())).unwrap(),
                    )
                    .execute()
                    .await
                    .map_err(|e| {
                        AppError::Sync(format!("Failed to insert into {}: {}", table_name, e))
                    })?;

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    if status.as_u16() == 404 {
                        return Err(AppError::Sync(format!(
                            "Table '{}' not accessible for insert. RLS policies may be blocking service role access. See SUPABASE_404_TROUBLESHOOTING.md",
                            table_name
                        )));
                    } else {
                        return Err(AppError::Sync(format!(
                            "Failed to insert data into {}: {} - {}",
                            table_name, status, text
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    async fn sync_single_change(&self, change: &PendingChange) -> AppResult<()> {
        match change.operation.as_str() {
            "INSERT" | "UPDATE" => {
                if let Some(data) = &change.record_data {
                    let json_data: Value = serde_json::from_str(data)
                        .map_err(|e| AppError::Sync(format!("Invalid JSON data: {}", e)))?;

                    let response = self
                        .client
                        .from(&change.table_name)
                        .upsert(serde_json::to_string(&json_data).unwrap())
                        .execute()
                        .await
                        .map_err(|e| AppError::Sync(format!("Upsert failed: {}", e)))?;

                    if !response.status().is_success() {
                        let status = response.status();
                        let text = response.text().await.unwrap_or_default();
                        if status.as_u16() == 404 {
                            return Err(AppError::Sync(format!(
                                "Table '{}' not accessible for {}. Check RLS policies and table permissions.",
                                change.table_name, change.operation
                            )));
                        } else {
                            return Err(AppError::Sync(format!(
                                "Upsert failed for table {}: {} - {}",
                                change.table_name, status, text
                            )));
                        }
                    }
                }
            }
            "DELETE" => {
                let response = self
                    .client
                    .from(&change.table_name)
                    .delete()
                    .eq("id", change.record_id.to_string())
                    .execute()
                    .await
                    .map_err(|e| AppError::Sync(format!("Delete failed: {}", e)))?;

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    if status.as_u16() == 404 {
                        return Err(AppError::Sync(format!(
                            "Table '{}' not accessible for DELETE or record not found (ID: {})",
                            change.table_name, change.record_id
                        )));
                    } else {
                        return Err(AppError::Sync(format!(
                            "Delete failed for table {}: {} - {}",
                            change.table_name, status, text
                        )));
                    }
                }
            }
            _ => {
                return Err(AppError::Sync(format!(
                    "Unknown operation: {}",
                    change.operation
                )));
            }
        }

        Ok(())
    }
}

pub async fn test_supabase_connection(url: &str, key: &str) -> AppResult<()> {
    let client = SupabaseClient::new(url, key)?;
    client.test_connection().await
}

pub async fn perform_full_sync(app_handle: &tauri::AppHandle) -> AppResult<()> {
    let state = app_handle.state::<AppState>();
    // wait for sync mutex to ensure only one sync at a time
    let _sync_guard = state.sync_mutex.lock().await;

    let settings = state.settings.read().await;

    let url = settings
        .supabase_url
        .as_ref()
        .ok_or_else(|| AppError::Sync("Supabase URL not configured".to_string()))?;
    let key = settings
        .supabase_key
        .as_ref()
        .ok_or_else(|| AppError::Sync("Supabase key not configured".to_string()))?;

    let client = SupabaseClient::new(url, key)?;

    // Ensure tables exist
    client.ensure_tables_exist().await?;

    let pool = &state.db_pool;

    // Emit sync status
    let _ = app_handle.emit("sync_status", "performing_full_sync");

    // Define tables with their specific query logic - only sync relevant tables
    let table_configs = [
        (
            "members",
            "SELECT * FROM members WHERE is_deleted = 0 OR is_deleted IS NULL",
        ),
        (
            "membership_types",
            "SELECT * FROM membership_types WHERE is_deleted = 0 OR is_deleted IS NULL",
        ),
        (
            "memberships",
            "SELECT * FROM memberships WHERE is_deleted = 0 OR is_deleted IS NULL",
        ),
    ];

    let mut sync_errors = Vec::new();

    for (table_name, query) in &table_configs {
        tracing::info!("Syncing table: {}", table_name);

        let rows = match sqlx::query(query).fetch_all(pool).await {
            Ok(rows) => rows,
            Err(e) => {
                let error_msg = format!("Failed to fetch data from {}: {}", table_name, e);
                tracing::error!("{}", error_msg);
                sync_errors.push(error_msg);
                continue;
            }
        };

        let mut data = Vec::new();
        for row in rows {
            let mut json_obj = serde_json::Map::new();

            for column in row.columns() {
                let column_name = column.name();
                let value = match column.type_info().name() {
                    "INTEGER" | "BIGINT" => {
                        if let Ok(val) = row.try_get::<i64, _>(column_name) {
                            serde_json::Value::Number(serde_json::Number::from(val))
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "REAL" | "FLOAT" => {
                        if let Ok(val) = row.try_get::<f64, _>(column_name) {
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(val)
                                    .unwrap_or(serde_json::Number::from(0)),
                            )
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "BOOLEAN" => {
                        if let Ok(val) = row.try_get::<bool, _>(column_name) {
                            serde_json::Value::Bool(val)
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    "DATE" | "DATETIME" | "TIMESTAMP" => {
                        // Handle date/datetime fields specifically
                        if let Ok(val) = row.try_get::<String, _>(column_name) {
                            if val.is_empty()
                                || val.trim().is_empty()
                                || val == "0000-00-00"
                                || val == "0000-00-00 00:00:00"
                            {
                                serde_json::Value::Null
                            } else {
                                serde_json::Value::String(val)
                            }
                        } else {
                            serde_json::Value::Null
                        }
                    }
                    _ => {
                        if let Ok(val) = row.try_get::<String, _>(column_name) {
                            // Convert empty strings to NULL for consistency
                            if val.is_empty() || val.trim().is_empty() {
                                serde_json::Value::Null
                            } else {
                                serde_json::Value::String(val)
                            }
                        } else {
                            serde_json::Value::Null
                        }
                    }
                };
                json_obj.insert(column_name.to_string(), value);
            }
            data.push(serde_json::Value::Object(json_obj));
        }

        match client.clear_and_populate_table(table_name, &data).await {
            Ok(()) => {
                tracing::info!(
                    "Successfully synced {} records from table {}",
                    data.len(),
                    table_name
                );
            }
            Err(e) => {
                let error_msg = format!("Failed to sync table {}: {}", table_name, e);
                tracing::error!("{}", error_msg);
                sync_errors.push(error_msg);
            }
        }
    }

    // Check if there were any errors during sync
    if !sync_errors.is_empty() {
        let error_summary = format!(
            "Full sync completed with {} errors: {}",
            sync_errors.len(),
            sync_errors.join("; ")
        );
        tracing::error!("{}", error_summary);
        let _ = app_handle.emit("sync_status", "full_sync_failed");
        return Err(AppError::Sync(error_summary));
    }

    // Clear pending changes after successful full sync
    if let Err(e) = sqlx::query("DELETE FROM pending_changes")
        .execute(pool)
        .await
    {
        let error_msg = format!("Failed to clear pending changes: {}", e);
        tracing::error!("{}", error_msg);
        let _ = app_handle.emit("sync_status", "full_sync_failed");
        return Err(AppError::Database(error_msg));
    }

    let _ = app_handle.emit("sync_status", "full_sync_completed");
    tracing::info!("Full sync completed successfully");

    Ok(())
}

pub async fn sync_pending_changes(app_handle: &tauri::AppHandle, instant: bool) -> AppResult<()> {
    let state = app_handle.state::<AppState>();
    // wait for sync mutex to ensure only one sync at a time
    let _sync_guard = state.sync_mutex.lock().await;
    let pool = &state.db_pool;

    let check_query = if !instant {
        "SELECT COUNT(*) FROM pending_changes WHERE status = 'pending'"
    } else {
        // Only sync newest changes. Others will be handled by periodic sync worker.
        "SELECT COUNT(*) FROM pending_changes WHERE status = 'pending' AND created_at > datetime('now', '-1 minute')"
    };
    // fast check
    let pending_count: i64 = sqlx::query_scalar(check_query)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count pending changes: {}", e)))?;

    if pending_count == 0 {
        return Ok(());
    }

    let settings = state.settings.read().await;

    if !settings.sync_enabled {
        return Ok(());
    }

    let url = settings
        .supabase_url
        .as_ref()
        .ok_or_else(|| AppError::Sync("Supabase URL not configured".to_string()))?;
    let key = settings
        .supabase_key
        .as_ref()
        .ok_or_else(|| AppError::Sync("Supabase key not configured".to_string()))?;

    tracing::debug!("Found {} pending changes to sync", pending_count);
    let client = SupabaseClient::new(url, key)?;

    let _ = app_handle.emit("sync_status", "syncing_changes");

    // Get pending changes
    let pending_query = if !instant {
        "SELECT * FROM pending_changes WHERE status = 'pending' ORDER BY created_at ASC LIMIT ?"
    } else {
        // Only sync newest changes. Others will be handled by periodic sync worker.
        "SELECT * FROM pending_changes WHERE status = 'pending' AND created_at > datetime('now', '-1 minute') ORDER BY created_at ASC LIMIT ?"
    };
    let pending_changes: Vec<PendingChange> = sqlx::query_as::<_, PendingChange>(pending_query)
        .bind(BATCH_SIZE)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to fetch pending changes: {}", e)))?;

    // update status to 'processing' to avoid duplicate processing
    let change_ids: Vec<i64> = pending_changes.iter().map(|c| c.id).collect();
    let update_query = format!(
        "UPDATE pending_changes SET status = 'processing' WHERE id IN ({})",
        change_ids
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    sqlx::query(&update_query)
        .execute(pool)
        .await
        .map_err(|e| {
            AppError::Database(format!(
                "Failed to update pending changes to processing: {}",
                e
            ))
        })?;

    let mut synced_count = 0;
    let mut failed_count = 0;

    for change in pending_changes {
        match client.sync_single_change(&change).await {
            Ok(()) => {
                // Delete successful change
                sqlx::query("DELETE FROM pending_changes WHERE id = ?")
                    .bind(change.id)
                    .execute(pool)
                    .await
                    .map_err(|e| {
                        AppError::Database(format!("Failed to delete synced change: {}", e))
                    })?;

                synced_count += 1;
                tracing::debug!(
                    "Successfully synced change {} for table {}",
                    change.id,
                    change.table_name
                );
            }
            Err(e) => {
                failed_count += 1;
                let new_retry_count = change.retry_count + 1;

                if new_retry_count >= MAX_RETRY_ATTEMPTS {
                    tracing::error!(
                        "Change {} failed max retries ({}): {}",
                        change.id,
                        MAX_RETRY_ATTEMPTS,
                        e
                    );
                    // Update to failed status
                    sqlx::query(
                        "UPDATE pending_changes SET retry_count = ?, last_error = ?, status = 'failed' WHERE id = ?",
                    )
                    .bind(new_retry_count)
                    .bind(format!("{}", e))
                    .bind(change.id)
                    .execute(pool)
                    .await
                    .map_err(|e| {
                        AppError::Database(format!("Failed to update to failed status: {}", e))
                    })?;
                } else {
                    tracing::warn!(
                        "Change {} failed (retry {}/{}): {}",
                        change.id,
                        new_retry_count,
                        MAX_RETRY_ATTEMPTS,
                        e
                    );
                    // Reset to pending status for retry
                    sqlx::query(
                        "UPDATE pending_changes SET retry_count = ?, last_error = ?, status = 'pending' WHERE id = ?",
                    )
                    .bind(new_retry_count)
                    .bind(format!("{}", e))
                    .bind(change.id)
                    .execute(pool)
                    .await
                    .map_err(|e| {
                        AppError::Database(format!("Failed to update retry count: {}", e))
                    })?;
                }
            }
        }
    }

    tracing::info!(
        "Sync completed: {} synced, {} failed",
        synced_count,
        failed_count
    );

    // Reset any stuck 'processing' records to 'pending' before sync (do this on the next change so it doesn't slow down the sync loop)
    // This handles cases where the app crashed while processing
    if let Err(e) = sqlx::query(
        "UPDATE pending_changes SET status = 'pending' WHERE status = 'processing' AND created_at < datetime('now', '-1 minute')"
    )
    .execute(pool)
    .await {
        tracing::warn!("Failed to reset stuck processing records: {}", e);
    }

    if failed_count > 0 {
        let _ = app_handle.emit("sync_status", "sync_partial_failure");
    } else {
        let _ = app_handle.emit("sync_status", "sync_success");
    }

    Ok(())
}

pub async fn get_pending_changes_info(
    app_handle: &tauri::AppHandle,
) -> AppResult<serde_json::Value> {
    let state = app_handle.state::<AppState>();
    let pool = &state.db_pool;

    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pending_changes")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to count pending changes: {}", e)))?;

    let failed_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM pending_changes WHERE status = 'failed'")
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to count failed changes: {}", e)))?;

    let oldest_change: Option<NaiveDateTime> = sqlx::query_scalar(
        "SELECT MIN(created_at) FROM pending_changes WHERE created_at IS NOT NULL",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to get oldest change: {}", e)))?
    .flatten();

    Ok(serde_json::json!({
        "total_count": total_count,
        "failed_count": failed_count,
        "oldest_change": oldest_change,
        "pending_count": total_count - failed_count
    }))
}

pub async fn spawn_sync_check_task(app_handle: tauri::AppHandle) {
    tracing::info!("Spawning periodic sync check task");
    let state = app_handle.state::<AppState>();
    let settings = state.settings.read().await;
    if !settings.sync_enabled {
        tracing::debug!("Periodic sync check task not started because sync is disabled!");
        return;
    }
    let sync_period_seconds = settings
        .sync_period_seconds
        .unwrap_or(SYNC_CHECK_INTERVAL_SECONDS);
    drop(settings);

    tracing::info!(
        "Starting sync worker with {} second intervals",
        sync_period_seconds
    );

    tokio::spawn(async move {
        let mut check_timer = interval(Duration::from_secs(sync_period_seconds));
        // IMPORTANT: Use Delay behavior to ensure previous sync completes before next one starts
        check_timer.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            check_timer.tick().await;
            let state = app_handle.state::<AppState>();

            match sync_pending_changes(&app_handle, false).await {
                Ok(()) => {
                    tracing::debug!("Periodic sync check completed successfully");
                }
                Err(e) => {
                    tracing::error!("Periodic sync check failed: {:?}", e);
                    let _ = app_handle.emit("sync_status", "sync_failed");

                    let pool = &state.db_pool;
                    // Reset any processing records back to pending on error
                    if let Err(reset_err) = sqlx::query(
                        "UPDATE pending_changes SET status = 'pending' WHERE status = 'processing'",
                    )
                    .execute(pool)
                    .await
                    {
                        tracing::error!(
                            "Failed to reset processing records after sync error: {}",
                            reset_err
                        );
                    }
                }
            }
        }
    });
}

pub async fn manual_trigger_sync(app_handle: tauri::AppHandle) -> AppResult<()> {
    sync_pending_changes(&app_handle, false).await
}

pub fn trigger_instant_sync(app_handle: &tauri::AppHandle) {
    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        // Slight delay to allow triggers to finish
        sleep(Duration::from_millis(2000)).await;
        if let Err(e) = sync_pending_changes(&app_handle_clone, true).await {
            tracing::debug!("Instant sync attempt failed (will retry later): {}", e);
        }
    });
}
