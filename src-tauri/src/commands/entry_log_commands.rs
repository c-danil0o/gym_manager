use crate::{
    dto::{
        EntryLogDisplay, EntryLogQueryParams, EntryStatus, MembershipInfo, PaginatedResponse,
        ScanPayload, ScanPayloadSingle, ScanProcessingResult,
    },
    error::Result as AppResult,
    models::Member,
    state::AppState,
    utils, AppError,
};
use chrono::{NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use sqlx::{Acquire, Row, Sqlite, SqliteConnection, Transaction};
use tauri::State;

async fn calculate_and_update_membership_status_if_needed(
    tx: &mut Transaction<'_, Sqlite>,
    membership_id: i64,
    current_status: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
    remaining_visits: i64,
    tz: &Tz,
) -> AppResult<String> {
    // Don't change suspended memberships
    if current_status == "suspended" {
        return Ok(current_status.to_string());
    }

    let now_date = Utc::now().with_timezone(tz).date_naive();
    let mut new_status = current_status.to_string();

    // Check for expiration conditions first
    if remaining_visits <= 0 || end_date < now_date {
        new_status = "expired".to_string();
    }

    // Handle pending -> active transition
    if current_status == "pending" && start_date <= now_date && new_status != "expired" {
        new_status = "active".to_string();
    }

    // Update database if status changed
    if new_status != current_status {
        let result = sqlx::query!(
            "UPDATE memberships SET status = ?, updated_at = ? WHERE id = ?",
            new_status,
            now_date,
            membership_id
        )
        .execute(&mut **tx)
        .await;
        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    tracing::error!(
                        "No rows affected when updating membership {} status",
                        membership_id
                    );
                    return Err(AppError::NotFound(
                        "Failed to update membership status".to_string(),
                    ));
                }
                tracing::info!(
                    "Updated membership {} status from {} to {}",
                    membership_id,
                    current_status,
                    new_status
                );
            }
            Err(e) => {
                tracing::error!(
                    "Database error updating membership {} status: {}",
                    membership_id,
                    e
                );
                return Err(AppError::Sqlx(e));
            }
        }
    }

    Ok(new_status)
}

async fn deny_entry(
    tx: &mut Transaction<'_, Sqlite>,
    tz: &Tz,
    member_id: Option<i64>,
    membership_id: Option<i64>,
    card_id: &str,
    status: EntryStatus,
    log_status: &str,
    message: &str,
    member_name: Option<&String>,
    membership: Option<&MembershipInfo>,
) -> AppResult<ScanProcessingResult> {
    log_entry_attempt(
        tx,
        tz,
        member_id,
        member_name.as_deref(),
        membership_id,
        card_id,
        log_status,
        message,
    )
    .await?;

    Ok(ScanProcessingResult {
        status,
        message: message.to_string(),
        member_name: member_name.cloned(),
        card_id: Some(card_id.to_string()),
        membership_type_name: membership.and_then(|m| m.membership_type_name.clone()),
        membership_end_date: membership.and_then(|m| m.membership_end_date),
        remaining_visits: membership.and_then(|m| m.membership_remaining_visits),
    })
}

#[tauri::command]
pub async fn process_scan(
    payload: ScanPayload,
    state: State<'_, AppState>,
) -> AppResult<ScanProcessingResult> {
    let mut scanned_card_id = payload.card_id.trim();

    if scanned_card_id.is_empty() {
        return Ok(ScanProcessingResult {
            status: EntryStatus::Error,
            message: "card_invalid".to_string(),
            member_name: None,
            card_id: None,
            membership_type_name: None,
            membership_end_date: None,
            remaining_visits: None,
        });
    }

    tracing::info!("Processing scan for card_id: {}", scanned_card_id);

    let mut conn = state.db_pool.acquire().await?;
    let mut tx = conn.begin().await.map_err(|e| {
        tracing::error!("Failed to start transaction: {}", e);
        AppError::Sqlx(e)
    })?;

    let gym_tz: Tz = state.settings.read().await.timezone.parse().map_err(|e| {
        tracing::error!("Failed to parse timezone from settings: {}", e);
        AppError::Config("Invalid gym timezone configuration.".to_string())
    })?;

    // Find Member by Card ID
    let member = match sqlx::query_as!(
        Member,
        "SELECT * FROM members WHERE (card_id = ?1 OR short_card_id = ?1) AND is_deleted = FALSE",
        scanned_card_id
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(member)) => member,
        Ok(None) => {
            let result = deny_entry(
                &mut tx,
                &gym_tz,
                None,
                None,
                scanned_card_id,
                EntryStatus::DeniedMemberNotFound,
                "denied_member_not_found",
                "member_not_found",
                None,
                None,
            )
            .await;
            // Commit transaction even for denied entries to save the log
            if let Err(e) = tx.commit().await {
                tracing::error!("Failed to commit transaction for denied entry: {}", e);
            }

            return result;
        }
        Err(e) => {
            tracing::error!("Database error finding member: {}", e);
            let _ = tx.rollback().await;
            return Err(AppError::Sqlx(e));
        }
    };

    let member_full_name = format!("{} {}", member.first_name, member.last_name);
    if let Some(card_id) = &member.card_id {
        scanned_card_id = card_id;
    } else {
        tracing::warn!("Member {} has no card ID assigned.", member_full_name);
    }
    utils::check_membership_statuses(&state).await?;

    // Get membership information
    let membership = match sqlx::query_as!(
        MembershipInfo,
        r#"
        SELECT
            ms.id AS membership_id,
            ms.member_id,
            ms.membership_type_id,
            ms.start_date as membership_start_date,
            ms.end_date as membership_end_date,
            ms.remaining_visits as membership_remaining_visits,
            ms.status as membership_status,
            ms.purchase_date as membership_purchase_date,
            mt.name AS membership_type_name,
            mt.enter_by AS membership_type_enter_by,
            '' AS member_first_name,
            '' AS member_last_name
        FROM
            memberships ms
        LEFT JOIN
            membership_types mt ON ms.membership_type_id = mt.id
        WHERE
            ms.member_id = ?
            AND (ms.is_deleted IS NULL OR ms.is_deleted = FALSE)
        ORDER BY
            CASE ms.status WHEN 'active' THEN 0 ELSE 1 END ASC,
            CASE ms.status WHEN 'pending' THEN 0 ELSE 1 END ASC,
            CASE WHEN ms.status = 'pending' THEN ms.start_date ELSE NULL END ASC,
            CASE WHEN ms.status = 'active' THEN ms.start_date ELSE NULL END DESC,
            ms.start_date DESC
        LIMIT 1;
        "#,
        member.id
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(membership)) => membership,
        Ok(None) => {
            let result = deny_entry(
                &mut tx,
                &gym_tz,
                Some(member.id),
                None,
                scanned_card_id,
                EntryStatus::DeniedNoMembership,
                "denied_no_membership",
                "no_membership",
                Some(&member_full_name),
                None,
            )
            .await;

            if let Err(e) = tx.commit().await {
                tracing::error!("Failed to commit transaction for denied entry: {}", e);
            }

            return result;
        }
        Err(e) => {
            tracing::error!("Database error finding membership: {}", e);
            let _ = tx.rollback().await;
            return Err(AppError::Sqlx(e));
        }
    };
    // Validate membership data integrity
    let (membership_id, current_status, start_date, end_date, remaining_visits) = match (
        membership.membership_id,
        membership.membership_status.as_ref(),
        membership.membership_start_date,
        membership.membership_end_date,
        membership.membership_remaining_visits,
    ) {
        (Some(id), Some(status), Some(s_date), Some(e_date), Some(rem_visits)) => {
            (id, status, s_date, e_date, rem_visits)
        }
        _ => {
            tracing::error!(
                "Invalid membership data for member {}: missing required fields",
                member_full_name
            );
            let result = deny_entry(
                &mut tx,
                &gym_tz,
                Some(member.id),
                membership.membership_id,
                scanned_card_id,
                EntryStatus::DeniedNoMembership,
                "denied_no_membership",
                "invalid_membership",
                Some(&member_full_name),
                Some(&membership),
            )
            .await;

            if let Err(e) = tx.commit().await {
                tracing::error!("Failed to commit transaction for denied entry: {}", e);
            }

            return result;
        }
    };
    // Calculate and update membership status
    let membership_status = match calculate_and_update_membership_status_if_needed(
        &mut tx,
        membership_id,
        current_status,
        start_date,
        end_date,
        remaining_visits,
        &gym_tz,
    )
    .await
    {
        Ok(status) => status,
        Err(e) => {
            tracing::error!("Failed to update membership status: {}", e);
            let _ = tx.rollback().await;
            return Err(e);
        }
    };

    // Check membership status and handle non-active cases
    match membership_status.as_str() {
        "expired" => {
            let (status, log_status, message) =
                if membership.membership_remaining_visits.unwrap_or(1) <= 0 {
                    (
                        EntryStatus::DeniedNoVisitsLeft,
                        "denied_no_visits_left",
                        String::from("no_visits_left"),
                    )
                } else {
                    (
                        EntryStatus::DeniedMembershipExpired,
                        "denied_membership_expired",
                        format!("expired_on|{:?}", end_date),
                    )
                };
            let result = deny_entry(
                &mut tx,
                &gym_tz,
                Some(member.id),
                Some(membership_id),
                scanned_card_id,
                status,
                log_status,
                message.as_str(),
                Some(&member_full_name),
                Some(&membership),
            )
            .await;

            if let Err(e) = tx.commit().await {
                tracing::error!("Failed to commit transaction for denied entry: {}", e);
            }

            return result;
        }
        "pending" => {
            let result = deny_entry(
                &mut tx,
                &gym_tz,
                Some(member.id),
                Some(membership_id),
                scanned_card_id,
                EntryStatus::DeniedMembershipNotActiveYet,
                "denied_membership_not_active_yet",
                &format!("pending|{:?}.", start_date),
                Some(&member_full_name),
                Some(&membership),
            )
            .await;

            if let Err(e) = tx.commit().await {
                tracing::error!("Failed to commit transaction for denied entry: {}", e);
            }

            return result;
        }
        "inactive" => {
            let result = deny_entry(
                &mut tx,
                &gym_tz,
                Some(member.id),
                Some(membership_id),
                scanned_card_id,
                EntryStatus::DeniedNoMembership,
                "denied_membership_inactive",
                "inactive",
                Some(&member_full_name),
                Some(&membership),
            )
            .await;

            if let Err(e) = tx.commit().await {
                tracing::error!("Failed to commit transaction for denied entry: {}", e);
            }

            return result;
        }
        "suspended" => {
            let result = deny_entry(
                &mut tx,
                &gym_tz,
                Some(member.id),
                Some(membership_id),
                scanned_card_id,
                EntryStatus::DeniedMembershipSuspended,
                "denied_membership_suspended",
                "suspended",
                Some(&member_full_name),
                Some(&membership),
            )
            .await;

            if let Err(e) = tx.commit().await {
                tracing::error!("Failed to commit transaction for denied entry: {}", e);
            }

            return result;
        }
        "active" => {
            tracing::info!("Member {} has an active membership.", member_full_name);
        }
        _ => {
            let result = deny_entry(
                &mut tx,
                &gym_tz,
                Some(member.id),
                Some(membership_id),
                scanned_card_id,
                EntryStatus::DeniedNoMembership,
                "invalid_status",
                &format!("Membership is currently {:?}.", membership_status),
                Some(&member_full_name),
                Some(&membership),
            )
            .await;

            if let Err(e) = tx.commit().await {
                tracing::error!("Failed to commit transaction for denied entry: {}", e);
            }

            return result;
        }
    }

    if let Some(enter_by_hours) = membership.membership_type_enter_by {
        let now = Utc::now().with_timezone(&gym_tz);
        let current_hour = now.time().hour() as i64;

        if current_hour >= enter_by_hours {
            let result = deny_entry(
                &mut tx,
                &gym_tz,
                Some(member.id),
                Some(membership_id),
                scanned_card_id,
                EntryStatus::DeniedAfterHours,
                "denied_after_hours",
                &format!("after_hours|{}", enter_by_hours),
                Some(&member_full_name),
                Some(&membership),
            )
            .await;

            if let Err(e) = tx.commit().await {
                tracing::error!("Failed to commit transaction for denied entry: {}", e);
            }

            return result;
        }
    } // Check if member is already checked in today

    let now_local = Utc::now().with_timezone(&gym_tz);
    let today_local = now_local.date_naive();

    let already_checked_in = match sqlx::query!(
        "SELECT COUNT(*) as count FROM entry_logs WHERE member_id = ? AND local_date = ? AND status = 'allowed'",
        member.id,
        today_local
    )
    .fetch_one(&mut *tx)
    .await
    {
      Ok(row) => row.count > 0,
      Err(e) => {
          tracing::error!("Database error checking existing entries: {}", e);
          let _ = tx.rollback().await;
          return Err(AppError::Sqlx(e));
      }
    };

    if already_checked_in {
        let result = deny_entry(
            &mut tx,
            &gym_tz,
            Some(member.id),
            Some(membership_id),
            scanned_card_id,
            EntryStatus::DeniedAlreadyCheckedIn,
            "denied_already_checked_in",
            "already_checked",
            Some(&member_full_name),
            Some(&membership),
        )
        .await;
        if let Err(e) = tx.commit().await {
            tracing::error!("Failed to commit transaction for denied entry: {}", e);
        }

        return result;
    }

    // Handle active membership - update visits and allow entry
    let new_visits = remaining_visits - 1;
    let new_status = if new_visits > 0 { "active" } else { "expired" };
    let now = Utc::now().naive_utc();

    let update_result = sqlx::query!(
        "UPDATE memberships SET remaining_visits = ?, updated_at = ?, status = ? WHERE id = ?",
        new_visits,
        now,
        new_status,
        membership_id
    )
    .execute(&mut *tx)
    .await;

    match update_result {
        Ok(query_result) => {
            if query_result.rows_affected() == 0 {
                tracing::error!(
                    "No rows affected when updating membership {} visits",
                    membership_id
                );
                let _ = tx.rollback().await;
                return Err(AppError::Database(
                    "Failed to update membership visits".to_string(),
                ));
            }
            tracing::info!(
                "Updated membership {} visits to {}",
                membership_id,
                new_visits
            );
        }
        Err(e) => {
            tracing::error!("Failed to update membership {}: {}", membership_id, e);
            let _ = tx.rollback().await;
            return Err(AppError::Database(format!(
                "Failed to update membership: {}",
                e
            )));
        }
    }
    // Log successful entry
    if let Err(e) = log_entry_attempt(
        &mut tx,
        &gym_tz,
        Some(member.id),
        Some(&member_full_name),
        Some(membership_id),
        scanned_card_id,
        "allowed",
        "allowed",
    )
    .await
    {
        tracing::error!("Failed to log successful entry: {}", e);
        let _ = tx.rollback().await;
        return Err(e);
    }

    // Commit transaction
    match tx.commit().await {
        Ok(_) => {
            tracing::info!(
                "Successfully processed entry for member: {}",
                member_full_name
            );
        }
        Err(e) => {
            tracing::error!("Failed to commit transaction: {}", e);
            return Err(AppError::Database(format!(
                "Failed to commit entry transaction: {}",
                e
            )));
        }
    }
    Ok(ScanProcessingResult {
        status: EntryStatus::Allowed,
        message: "allowed".to_string(),
        member_name: Some(member_full_name),
        card_id: member.card_id.clone(),
        membership_type_name: membership.membership_type_name.clone(),
        membership_end_date: membership.membership_end_date,
        remaining_visits: Some(new_visits),
    })
}

#[tauri::command]
pub async fn process_scan_single(
    payload: ScanPayloadSingle,
    state: State<'_, AppState>,
) -> AppResult<ScanProcessingResult> {
    let mut conn = state.db_pool.acquire().await?;
    let mut tx = conn.begin().await.map_err(|e| {
        tracing::error!("Failed to start transaction: {}", e);
        AppError::Sqlx(e)
    })?;

    let gym_tz: Tz = state.settings.read().await.timezone.parse().map_err(|e| {
        tracing::error!("Failed to parse timezone from settings: {}", e);
        AppError::Config("Invalid gym timezone configuration.".to_string())
    })?;

    match payload.card_id {
        Some(card_id) if !card_id.is_empty() => {
            let member = match sqlx::query_as!(
              Member,
              "SELECT * FROM members WHERE (card_id = ?1 OR short_card_id = ?1) AND is_deleted = FALSE",
              card_id
          )
          .fetch_optional(&mut *tx)
          .await?
          {
              Some(member) => member,
              None => {
                  let result = deny_entry(
                      &mut tx,
                      &gym_tz,
                      None,
                      None,
                      card_id.as_str(),
                      EntryStatus::DeniedMemberNotFound,
                      "denied_member_not_found",
                      "member_not_found",
                      None,
                      None,
                  )
                  .await;
                  if let Err(e) = tx.commit().await {
                      tracing::error!("Failed to commit transaction for denied entry: {}", e);
                  }
                  return result;
              }
          };
            let card_id = member.card_id.as_deref().unwrap_or(&card_id);
            let member_full_name = format!("{} {}", member.first_name, member.last_name);

            if let Err(e) = log_entry_attempt(
                &mut tx,
                &gym_tz,
                Some(member.id),
                Some(&member_full_name),
                None,
                card_id,
                "allowed_single",
                "allowed_single",
            )
            .await
            {
                tracing::error!("Failed to log successful entry: {}", e);
                let _ = tx.rollback().await;
                return Err(e);
            }

            // Commit transaction
            match tx.commit().await {
                Ok(_) => {
                    tracing::info!(
                        "Successfully processed entry for member: {}",
                        member_full_name
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to commit transaction: {}", e);
                    return Err(AppError::Database(format!(
                        "Failed to commit entry transaction: {}",
                        e
                    )));
                }
            }

            Ok(ScanProcessingResult {
                status: EntryStatus::AllowedSingle,
                message: "allowed_single".to_string(),
                member_name: Some(member_full_name),
                card_id: Some(card_id.to_string()),
                membership_type_name: None,
                membership_end_date: None,
                remaining_visits: None,
            })
        }

        _ => {
            if payload.first_name.is_none() && payload.last_name.is_none() {
                return Ok(ScanProcessingResult {
                    status: EntryStatus::Error,
                    message: "card_invalid".to_string(),
                    member_name: None,
                    card_id: None,
                    membership_type_name: None,
                    membership_end_date: None,
                    remaining_visits: None,
                });
            }
            let member_full_name = format!(
                "{} {}",
                payload.first_name.as_deref().unwrap_or(""),
                payload.last_name.as_deref().unwrap_or("")
            );

            if let Err(e) = log_entry_attempt(
                &mut tx,
                &gym_tz,
                None,
                Some(&member_full_name),
                None,
                "",
                "allowed_single",
                "allowed_single",
            )
            .await
            {
                tracing::error!("Failed to log successful entry: {}", e);
                let _ = tx.rollback().await;
                return Err(e);
            }

            // Commit transaction
            match tx.commit().await {
                Ok(_) => {
                    tracing::info!(
                        "Successfully processed entry for member: {}",
                        member_full_name
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to commit transaction: {}", e);
                    return Err(AppError::Database(format!(
                        "Failed to commit entry transaction: {}",
                        e
                    )));
                }
            }
            Ok(ScanProcessingResult {
                status: EntryStatus::AllowedSingle,
                message: "allowed_single".to_string(),
                member_name: Some(member_full_name),
                card_id: None,
                membership_type_name: None,
                membership_end_date: None,
                remaining_visits: None,
            })
        }
    }
}

// Helper to log entry attempts
async fn log_entry_attempt(
    tx: &mut Transaction<'_, Sqlite>,
    tz: &Tz,
    member_id: Option<i64>,
    member_name: Option<&String>,
    membership_id: Option<i64>,
    card_id_scanned: &str,
    status: &str,
    notes: &str,
) -> AppResult<()> {
    let now = Utc::now().naive_utc();
    let now_local = Utc::now().with_timezone(tz);
    let local_date = now_local.date_naive();
    sqlx::query!(
        r#"
        INSERT INTO entry_logs (member_id, membership_id, member_name, card_id, entry_time, status, notes, local_date)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        member_id,
        membership_id,
        member_name,
        card_id_scanned,
        now,
        status,
        notes,
        local_date
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn build_where_clause_and_params(params: &EntryLogQueryParams) -> (String, Vec<String>) {
    let mut where_conditions = Vec::new();
    let mut sql_params = Vec::new();

    let search_term = params
        .search_string
        .as_ref()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty());

    if let Some(term) = &search_term {
        let like_pattern = format!("'%{}%'", term);
        let search_query = format!(
            "LOWER(el.member_name) LIKE {val} OR el.card_id LIKE {val}",
            val = like_pattern
        );
        where_conditions.push(search_query);
    }

    if let Some(filter_fields) = &params.filter_fields {
        let mut filter_conditions = Vec::new();
        let filter_query;
        for field in filter_fields {
            match field.field.as_str() {
                "status" => {
                    filter_conditions.push(format!(
                        "COALESCE(el.status, '') IN ('{}')",
                        field
                            .value
                            .split(',')
                            .map(|s| s.trim())
                            .collect::<Vec<_>>()
                            .join("', '")
                    ));
                }
                _ => {}
            }
        }

        if !filter_conditions.is_empty() {
            filter_query = filter_conditions.join(" AND ");
            where_conditions.push(filter_query);
        }
    }

    // Date range search
    if let Some(date_from) = params.date_from {
        where_conditions.push("DATE(el.entry_time) >= ?".to_string());
        sql_params.push(date_from.to_string());
    }

    if let Some(date_to) = params.date_to {
        where_conditions.push("DATE(el.entry_time) <= ?".to_string());
        sql_params.push(date_to.to_string());
    }

    let where_clause = if where_conditions.is_empty() {
        "1=1".to_string()
    } else {
        where_conditions.join(" AND ")
    };

    (where_clause, sql_params)
}

fn build_order_clause(params: &EntryLogQueryParams) -> String {
    let order_by = params.order_by.as_deref().unwrap_or("entry_time");
    let direction = params.order_direction.as_deref().unwrap_or("desc");

    let valid_order_fields = ["entry_time", "member_name", "status", "card_id"];
    let order_field = if valid_order_fields.contains(&order_by) {
        match order_by {
            "member_name" => "el.member_name",
            "entry_time" => "el.entry_time",
            "status" => "el.status",
            "card_id" => "el.card_id",
            _ => "el.entry_time",
        }
    } else {
        "el.entry_time"
    };

    let order_direction = if direction.to_lowercase() == "asc" {
        "ASC"
    } else {
        "DESC"
    };

    format!("ORDER BY {} {}", order_field, order_direction)
}

async fn get_total_count(
    conn: &mut SqliteConnection,
    where_clause: &str,
    params: &[String],
) -> AppResult<i64> {
    let count_query = format!(
        r#"
        SELECT COUNT(*) as total
        FROM entry_logs el
        LEFT JOIN memberships ms ON el.membership_id = ms.id
        LEFT JOIN membership_types mt ON ms.membership_type_id = mt.id
        WHERE {}
        "#,
        where_clause
    );

    let mut query = sqlx::query_scalar::<_, i64>(&count_query);
    for param in params {
        query = query.bind(param);
    }

    let total = query.fetch_one(conn).await?;
    Ok(total)
}

#[tauri::command]
pub async fn get_entry_logs(
    search_params: EntryLogQueryParams,
    state: State<'_, AppState>,
) -> AppResult<PaginatedResponse<EntryLogDisplay>> {
    let mut conn = state.db_pool.acquire().await?;

    // Validate and set defaults for pagination
    let page = search_params.page.unwrap_or(1).max(1);
    let per_page = search_params.per_page.unwrap_or(50).min(100).max(1);
    let offset = (page - 1) * per_page;

    let (where_clause, params) = build_where_clause_and_params(&search_params);

    let total_count = get_total_count(&mut conn, &where_clause, &params).await?;

    let total_pages = ((total_count as f64) / (per_page as f64)).ceil() as i32;

    let order_clause = build_order_clause(&search_params);

    let main_query = format!(
        r#"
        SELECT
            el.id,
            el.member_id,
            el.membership_id,
            el.member_name,
            mt.name as membership_type_name,
            ms.remaining_visits as visits_left,
            el.card_id,
            el.entry_time,
            el.status,
            el.notes
        FROM entry_logs el
        LEFT JOIN memberships ms ON el.membership_id = ms.id
        LEFT JOIN membership_types mt ON ms.membership_type_id = mt.id
        WHERE {}
        {}
        LIMIT ? OFFSET ?
        "#,
        where_clause, order_clause
    );

    let mut query = sqlx::query_as::<_, EntryLogDisplay>(&main_query);

    for param in &params {
        query = query.bind(param);
    }

    query = query.bind(per_page as i64).bind(offset as i64);

    let entries = query.fetch_all(&mut *conn).await?;

    Ok(PaginatedResponse {
        data: entries,
        total: total_count,
        page,
        per_page,
        total_pages: total_pages as i64,
    })
}

#[tauri::command]
pub async fn get_recent_entry_logs(
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> AppResult<Vec<EntryLogDisplay>> {
    let limit = limit.unwrap_or(100).min(500).max(1);
    let mut conn = state.db_pool.acquire().await?;

    let entries = sqlx::query_as!(
        EntryLogDisplay,
        r#"
        SELECT
            el.id as 'id!',
            el.member_id,
            el.membership_id,
            el.member_name,
            mt.name as membership_type_name,
            ms.remaining_visits as visits_left,
            el.card_id,
            el.entry_time,
            el.status,
            el.notes
        FROM entry_logs el
        LEFT JOIN memberships ms ON el.membership_id = ms.id
        LEFT JOIN membership_types mt ON ms.membership_type_id = mt.id
        ORDER BY el.entry_time DESC
        LIMIT ?
        "#,
        limit
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok(entries)
}

// Helper function to get entry logs for a specific member
#[tauri::command]
pub async fn get_member_entry_logs(
    member_id: i64,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> AppResult<Vec<EntryLogDisplay>> {
    let limit = limit.unwrap_or(50).min(200).max(1);
    let mut conn = state.db_pool.acquire().await?;

    let entries = sqlx::query_as!(
        EntryLogDisplay,
        r#"
        SELECT
            el.id as 'id!',
            el.member_id,
            el.membership_id,
            el.member_name,
            ms.remaining_visits as visits_left,
            mt.name as membership_type_name,
            el.card_id,
            el.entry_time,
            el.status,
            el.notes
        FROM entry_logs el
        LEFT JOIN memberships ms ON el.membership_id = ms.id
        LEFT JOIN membership_types mt ON ms.membership_type_id = mt.id
        WHERE el.member_id = ?
        ORDER BY el.entry_time DESC
        LIMIT ?
        "#,
        member_id,
        limit
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok(entries)
}

// Statistics function for dashboard
#[tauri::command]
pub async fn get_entry_logs_stats(
    date_from: Option<NaiveDate>,
    date_to: Option<NaiveDate>,
    state: State<'_, AppState>,
) -> AppResult<serde_json::Value> {
    let mut conn = state.db_pool.acquire().await?;

    let mut where_conditions = vec!["1=1"];
    let mut params = Vec::new();

    if let Some(from_date) = date_from {
        where_conditions.push("DATE(el.entry_time) >= ?");
        params.push(from_date.to_string());
    }

    if let Some(to_date) = date_to {
        where_conditions.push("DATE(el.entry_time) <= ?");
        params.push(to_date.to_string());
    }

    let where_clause = where_conditions.join(" AND ");

    let stats_query = format!(
        r#"
        SELECT
            COUNT(*) as total_entries,
            SUM(CASE WHEN el.status = 'allowed' THEN 1 ELSE 0 END) as allowed_entries,
            SUM(CASE WHEN el.status LIKE 'denied%' THEN 1 ELSE 0 END) as denied_entries,
            COUNT(DISTINCT el.member_id) as unique_members,
            COUNT(DISTINCT DATE(el.entry_time)) as unique_days
        FROM entry_logs el
        WHERE {}
        "#,
        where_clause
    );

    let mut query = sqlx::query(&stats_query);
    for param in &params {
        query = query.bind(param);
    }

    let row = query.fetch_one(&mut *conn).await?;

    let stats = serde_json::json!({
        "total_entries": row.get::<i64, _>("total_entries"),
        "allowed_entries": row.get::<i64, _>("allowed_entries"),
        "denied_entries": row.get::<i64, _>("denied_entries"),
        "unique_members": row.get::<i64, _>("unique_members"),
        "unique_days": row.get::<i64, _>("unique_days"),
        "success_rate": if row.get::<i64, _>("total_entries") > 0 {
            (row.get::<i64, _>("allowed_entries") as f64 / row.get::<i64, _>("total_entries") as f64) * 100.0
        } else {
            0.0
        }
    });

    Ok(stats)
}

#[tauri::command]
pub async fn delete_entry_logs(
    period: Option<i64>, // number ofa recent months of logs to keep
    state: State<'_, AppState>,
) -> AppResult<()> {
    let mut conn = state.db_pool.acquire().await?;
    if period.is_none() {
        return Err(AppError::Validation("Period must be specified".to_string()));
    }
    let period = period.unwrap();

    // If period is 0  delete all logs
    if period == 0 {
        sqlx::query!("DELETE FROM entry_logs")
            .execute(&mut *conn)
            .await?;
        return Ok(());
    }
    if period < 0 || period > 60 {
        return Err(AppError::Validation(
            "Period must be between 0 and 60 months".to_string(),
        ));
    }

    // Calculate the date threshold
    let threshold_date = Utc::now()
        .naive_utc()
        .checked_sub_signed(chrono::Duration::days(period * 30))
        .ok_or(AppError::Validation("Invalid period".to_string()))?;

    // Delete logs older than the threshold date
    sqlx::query!(
        "DELETE FROM entry_logs WHERE entry_time < ?",
        threshold_date
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn delete_entry_log(entry_log_id: i64, state: State<'_, AppState>) -> AppResult<()> {
    let mut conn = state.db_pool.acquire().await?;

    sqlx::query!("DELETE FROM entry_logs WHERE id = ?", entry_log_id)
        .execute(&mut *conn)
        .await?;

    Ok(())
}
