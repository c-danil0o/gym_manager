use std::time::Duration;

use crate::{
    error::{AppError, Result as AppResult},
    models::CronCheck,
    AppState,
};

const MEMBERSHIP_CHECK_INTERVAL_MINUTES: u64 = 15;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Local;
use tauri::Manager;
use tokio::time::interval;

/// Hashes a password using Argon2.
pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Config(format!("Failed to hash password: {}", e)))?
        .to_string();

    Ok(password_hash)
}

#[allow(dead_code)]
pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Config(format!("Invalid password hash format: {}", e)))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

async fn load_last_checked_membership_date(
    app_state: &tauri::State<'_, AppState>,
) -> AppResult<chrono::NaiveDateTime> {
    let check = sqlx::query_as!(
          CronCheck,
          "SELECT id as `id!`, last_check_time, check_type, status, created_at, updated_at FROM cron_checks WHERE check_type = 'membership' ORDER BY last_check_time DESC LIMIT 1",
      )
      .fetch_optional(&app_state.db_pool)
      .await?;

    match check {
        Some(check) => Ok(check.last_check_time),
        None => Ok(chrono::NaiveDateTime::from_timestamp(0, 0)),
    }
}

async fn update_membership_statuses(app_state: &tauri::State<'_, AppState>) -> AppResult<i64> {
    let today = Local::now().date_naive();
    let mut tx = app_state.db_pool.begin().await?;

    // 1. Update pending memberships to active (start_date <= today)
    let pending_query = r#"
        UPDATE memberships
        SET status = 'active',
            updated_at = CURRENT_TIMESTAMP
        WHERE status = 'pending'
        AND start_date <= ?
        AND is_deleted = FALSE
    "#;

    let pending_result = sqlx::query(pending_query)
        .bind(today)
        .execute(&mut *tx)
        .await?;

    let pending_to_active_count = pending_result.rows_affected() as i64;

    let expired_query = r#"
        UPDATE memberships
        SET status = 'expired',
            updated_at = CURRENT_TIMESTAMP
        WHERE status = 'active'
        AND end_date < ?
        AND is_deleted = FALSE
    "#;

    let expired_result = sqlx::query(expired_query)
        .bind(today)
        .execute(&mut *tx)
        .await?;

    let active_to_expired_count = expired_result.rows_affected() as i64;
    // Commit the transaction
    tx.commit().await?;

    let total_updated = pending_to_active_count + active_to_expired_count;

    // Log the results
    if total_updated > 0 {
        println!("📊 Membership Status Update Summary:");
        println!("   • Pending → Active: {}", pending_to_active_count);
        println!("   • Active → Expired: {}", active_to_expired_count);
        println!("   • Total Updated: {}", total_updated);
    } else {
        println!("✅ All membership statuses are already current");
    }

    Ok(total_updated)
}

async fn save_last_checked_membership_date(
    app_state: &tauri::State<'_, AppState>,
    date: chrono::NaiveDateTime,
    status: &str,
) -> AppResult<()> {
    // craete if not exists
    sqlx::query!(
        "INSERT INTO cron_checks (last_check_time, check_type, status, created_at) VALUES (?, 'membership', ?, CURRENT_TIMESTAMP)
         ON CONFLICT(check_type) DO UPDATE SET last_check_time = excluded.last_check_time, status = excluded.status, updated_at = CURRENT_TIMESTAMP",
        date,
        status
    )
    .execute(&app_state.db_pool)
    .await?;

    // Try to get the lock without waiting
    if let Ok(mut last_check) = app_state.last_membership_check.try_write() {
        *last_check = Some(date);
    } else {
        tracing::warn!(
            "Could not acquire write lock on last_membership_check, skipping in-memory update"
        );
    }
    Ok(())
}

pub async fn check_membership_statuses(app_state: &tauri::State<'_, AppState>) -> AppResult<()> {
    let today = Local::now().naive_local();
    let last_check = app_state.last_membership_check.read().await;
    let last_checked = last_check
        .clone()
        .unwrap_or(load_last_checked_membership_date(&app_state).await?);
    drop(last_check);

    if last_checked.date() < today.date() {
        let updated_count = update_membership_statuses(&app_state).await;
        match updated_count {
            Ok(count) => {
                if count > 0 {
                    save_last_checked_membership_date(&app_state, today, "success").await?;
                    println!("✅ Membership statuses updated successfully.");
                } else {
                    println!("No membership statuses needed updating.");
                }
            }
            Err(e) => {
                save_last_checked_membership_date(&app_state, today, "fail").await?;
                println!("❌ Error updating membership statuses: {}", e);
                return Err(AppError::MembershipCheckFailed(
                    "Failed to update membership statuses!".to_string(),
                ));
            }
        }
    } else {
        save_last_checked_membership_date(&app_state, today, "no_changes").await?;
        println!("Membership statuses already checked today.");
    }

    Ok(())
}

pub fn spawn_membership_check_task(app_handle: tauri::AppHandle) {
    tracing::info!(
        "Spawning periodic check task for membership status update (Interval: {} minutes)",
        MEMBERSHIP_CHECK_INTERVAL_MINUTES,
    );
    tokio::spawn(async move {
        let check_interval_duration = Duration::from_secs(MEMBERSHIP_CHECK_INTERVAL_MINUTES * 60);
        let mut check_timer = interval(check_interval_duration);
        loop {
            check_timer.tick().await;
            tracing::debug!("Running periodic membership status check...");

            // Get the state inside the task
            let app_state = app_handle.state::<AppState>();
            if let Err(e) = check_membership_statuses(&app_state).await {
                tracing::error!("Error in periodic membership check: {:?}", e);
            }
        }
    });
}
