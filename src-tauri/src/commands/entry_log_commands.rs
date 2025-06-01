use crate::{
    dto::{
        EntryLogDisplay, EntryLogQueryParams, EntryLogSearchResult, EntryStatus, MembershipInfo,
        ScanPayload, ScanProcessingResult,
    },
    error::Result as AppResult,
    models::Member,
    state::AppState,
    AppError,
};
use chrono::{NaiveDate, Timelike, Utc};
use chrono_tz::Tz;
use sqlx::{Row, SqliteConnection};
use tauri::State;

const GYM_TIMEZONE: &str = "Europe/Belgrade";

async fn calculate_and_update_membership_status_if_needed(
    conn: &mut SqliteConnection,
    membership_id: i64,
    current_status: &str,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    remaining_visits: Option<i64>,
) -> AppResult<String> {
    // Don't change suspended memberships
    if current_status == "suspended" {
        return Ok(current_status.to_string());
    }

    let now_date = Utc::now().date_naive();
    let mut new_status = current_status.to_string();

    // Check for expiration conditions first
    if remaining_visits.map_or(false, |v| v <= 0) {
        new_status = "expired".to_string();
    } else if let Some(ed) = end_date {
        if ed < now_date {
            new_status = "expired".to_string();
        }
    } else if remaining_visits.is_none() && end_date.is_none() {
        new_status = "expired".to_string();
    }

    // Handle pending -> active transition
    if current_status == "pending" && start_date <= now_date && new_status != "expired" {
        new_status = if end_date.is_some() {
            "active"
        } else {
            "inactive"
        }
        .to_string();
    }

    // Update database if status changed
    if new_status != current_status {
        sqlx::query!(
            "UPDATE memberships SET status = ?, updated_at = ? WHERE id = ?",
            new_status,
            now_date,
            membership_id
        )
        .execute(conn)
        .await?;
    }

    Ok(new_status)
}

async fn deny_entry(
    conn: &mut SqliteConnection,
    member_id: Option<i64>,
    membership_id: Option<i64>,
    card_id: &str,
    status: EntryStatus,
    log_status: &str,
    message: &str,
    member_name: Option<String>,
    membership: Option<&MembershipInfo>,
) -> AppResult<ScanProcessingResult> {
    log_entry_attempt(conn, member_id, membership_id, card_id, log_status, message).await?;

    Ok(ScanProcessingResult {
        status,
        message: message.to_string(),
        member_name,
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
            message: "Card ID cannot be empty.".to_string(),
            member_name: None,
            card_id: None,
            membership_type_name: None,
            membership_end_date: None,
            remaining_visits: None,
        });
    }

    tracing::info!("Processing scan for card_id: {}", scanned_card_id);

    let mut conn = state.db_pool.acquire().await?;

    // Find Member by Card ID
    let member = match sqlx::query_as!(
        Member,
        "SELECT * FROM members WHERE (card_id = ?1 OR short_card_id = ?1) AND is_deleted = FALSE",
        scanned_card_id
    )
    .fetch_optional(&mut *conn)
    .await?
    {
        Some(member) => member,
        None => {
            return deny_entry(
                &mut conn,
                None,
                None,
                scanned_card_id,
                EntryStatus::DeniedMemberNotFound,
                "denied_member_not_found",
                "Member not found for this card ID.",
                None,
                None,
            )
            .await;
        }
    };

    let member_full_name = format!("{} {}", member.first_name, member.last_name);
    if let Some(card_id) = &member.card_id {
        scanned_card_id = card_id;
    } else {
        tracing::warn!("Member {} has no card ID assigned.", member_full_name);
    }

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
    .fetch_optional(&mut *conn)
    .await?
    {
        Some(membership) => membership,
        None => {
            return deny_entry(
                &mut conn,
                Some(member.id),
                None,
                scanned_card_id,
                EntryStatus::DeniedNoMembership,
                "denied_no_membership",
                "No active or pending membership found for this member.",
                Some(member_full_name),
                None,
            )
            .await;
        }
    };

    // Validate membership data integrity
    let (membership_id, current_status, start_date) = match (
        membership.membership_id,
        membership.membership_status.as_ref(),
        membership.membership_start_date,
    ) {
        (Some(id), Some(status), Some(date)) => (id, status, date),
        _ => {
            return deny_entry(
                &mut conn,
                Some(member.id),
                None,
                scanned_card_id,
                EntryStatus::DeniedNoMembership,
                "denied_no_membership",
                "Membership information is incomplete.",
                Some(member_full_name),
                Some(&membership),
            )
            .await;
        }
    };

    // Calculate and update membership status
    let membership_status = calculate_and_update_membership_status_if_needed(
        &mut conn,
        membership_id,
        current_status,
        start_date,
        membership.membership_end_date,
        membership.membership_remaining_visits,
    )
    .await?;

    // Check membership status and handle non-active cases
    match membership_status.as_str() {
        "expired" => {
            let (status, log_status, message) =
                if membership.membership_remaining_visits.unwrap_or(1) <= 0 {
                    (
                        EntryStatus::DeniedNoVisitsLeft,
                        "denied_no_visits_left",
                        String::from("Membership has no visits remaining."),
                    )
                } else {
                    (
                        EntryStatus::DeniedMembershipExpired,
                        "denied_membership_expired",
                        format!(
                            "Membership expired on {:?}.",
                            membership.membership_end_date
                        ),
                    )
                };
            return deny_entry(
                &mut conn,
                Some(member.id),
                Some(membership_id),
                scanned_card_id,
                status,
                log_status,
                message.as_str(),
                Some(member_full_name),
                Some(&membership),
            )
            .await;
        }
        "pending" => {
            return deny_entry(
                &mut conn,
                Some(member.id),
                Some(membership_id),
                scanned_card_id,
                EntryStatus::DeniedMembershipNotActiveYet,
                "denied_membership_not_active_yet",
                &format!("Membership not active yet. Starts on {:?}.", start_date),
                Some(member_full_name),
                Some(&membership),
            )
            .await;
        }
        "inactive" => {
            return deny_entry(
                &mut conn,
                Some(member.id),
                Some(membership_id),
                scanned_card_id,
                EntryStatus::DeniedNoMembership,
                "denied_membership_inactive",
                "Membership is inactive.",
                Some(member_full_name),
                Some(&membership),
            )
            .await;
        }
        "suspended" => {
            return deny_entry(
                &mut conn,
                Some(member.id),
                Some(membership_id),
                scanned_card_id,
                EntryStatus::DeniedMembershipSuspended,
                "denied_membership_suspended",
                "Membership is currently suspended.",
                Some(member_full_name),
                Some(&membership),
            )
            .await;
        }
        "active" => {
            tracing::info!("Member {} has an active membership.", member_full_name);
        }
        _ => {
            return deny_entry(
                &mut conn,
                Some(member.id),
                Some(membership_id),
                scanned_card_id,
                EntryStatus::DeniedNoMembership,
                "denied_membership_invalid_status",
                &format!("Membership is currently {:?}.", membership_status),
                Some(member_full_name),
                Some(&membership),
            )
            .await;
        }
    }

    if let Some(enter_by_hours) = membership.membership_type_enter_by {
        // Check if entry is allowed based on enter_by_hours
        let gym_tz: Tz = GYM_TIMEZONE.parse().map_err(|e| {
            tracing::error!("Failed to parse GYM_TIMEZONE_STR: {}", e);
            AppError::Config("Invalid gym timezone configuration.".to_string())
        })?;

        let now = Utc::now().with_timezone(&gym_tz);

        // Get the current hour
        let current_hour = now.time().hour() as i64;

        if current_hour >= enter_by_hours {
            return deny_entry(
                &mut conn,
                Some(member.id),
                Some(membership_id),
                scanned_card_id,
                EntryStatus::DeniedAfterHours,
                "denied_after_hours",
                &format!("Entry not allowed after {}:00.", enter_by_hours),
                Some(member_full_name),
                Some(&membership),
            )
            .await;
        }
    }
    // Check if member is already checked in today
    let today = Utc::now().date_naive();
    let already_checked_in = sqlx::query!(
        "SELECT COUNT(*) as count FROM entry_logs WHERE member_id = ? AND DATE(entry_time) = ? AND status = 'allowed'",
        member.id,
        today
    )
    .fetch_one(&mut *conn)
    .await?
    .count
        > 0;

    if already_checked_in {
        return deny_entry(
            &mut conn,
            Some(member.id),
            Some(membership_id),
            scanned_card_id,
            EntryStatus::DeniedAlreadyCheckedIn,
            "denied_already_checked_in",
            "Member has already checked in today.",
            Some(member_full_name),
            Some(&membership),
        )
        .await;
    }

    // Handle active membership - update visits and allow entry
    let current_visits = membership.membership_remaining_visits.unwrap_or(0);
    let new_visits = current_visits - 1;
    let new_status = if new_visits > 0 { "active" } else { "expired" };
    let now = Utc::now().naive_utc();

    let update_result = sqlx::query!(
        "UPDATE memberships SET remaining_visits = ?, updated_at = ?, status = ? WHERE id = ?",
        new_visits,
        now,
        new_status,
        membership_id
    )
    .execute(&mut *conn)
    .await;

    match update_result {
        Ok(_) => tracing::info!(
            "Updated membership {} visits to {}",
            membership_id,
            new_visits
        ),
        Err(e) => {
            tracing::error!("Failed to update membership {}: {}", membership_id, e);
            return deny_entry(
                &mut conn,
                Some(member.id),
                Some(membership_id),
                scanned_card_id,
                EntryStatus::Error,
                "error",
                "Failed to update membership visits.",
                Some(member_full_name),
                Some(&membership),
            )
            .await;
        }
    }

    // Log successful entry
    log_entry_attempt(
        &mut conn,
        Some(member.id),
        Some(membership_id),
        scanned_card_id,
        "allowed",
        "Entry granted.",
    )
    .await?;

    Ok(ScanProcessingResult {
        status: EntryStatus::Allowed,
        message: "Entry Allowed!".to_string(),
        member_name: Some(member_full_name),
        card_id: member.card_id.clone(),
        membership_type_name: membership.membership_type_name.clone(),
        membership_end_date: membership.membership_end_date,
        remaining_visits: Some(new_visits),
    })
}

// Helper to log entry attempts
async fn log_entry_attempt(
    conn: &mut SqliteConnection,
    member_id: Option<i64>,
    membership_id: Option<i64>,
    card_id_scanned: &str,
    status: &str,
    notes: &str,
) -> AppResult<()> {
    let now = Utc::now().naive_utc();
    sqlx::query!(
        r#"
        INSERT INTO entry_logs (member_id, membership_id, card_id, entry_time, status, notes)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
        member_id,
        membership_id,
        card_id_scanned,
        now,
        status,
        notes
    )
    .execute(conn)
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
        let search_query = format!("(LOWER(m.first_name) LIKE {val} OR LOWER(m.last_name) LIKE {val} OR LOWER(m.first_name || ' ' || m.last_name) LIKE {val}) OR m.card_id LIKE {val}", val = like_pattern);
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
            "member_name" => "(m.first_name || ' ' || m.last_name)",
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
        LEFT JOIN members m ON el.member_id = m.id
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
) -> AppResult<EntryLogSearchResult> {
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
            CASE
                WHEN m.id IS NOT NULL THEN (m.first_name || ' ' || m.last_name)
                ELSE NULL
            END as member_name,
            mt.name as membership_type_name,
            el.card_id,
            el.entry_time,
            el.status,
            el.notes
        FROM entry_logs el
        LEFT JOIN members m ON el.member_id = m.id
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

    Ok(EntryLogSearchResult {
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
            CASE
                WHEN m.id IS NOT NULL THEN (m.first_name || ' ' || m.last_name)
                ELSE NULL
            END as member_name,
            mt.name as membership_type_name,
            el.card_id,
            el.entry_time,
            el.status,
            el.notes
        FROM entry_logs el
        LEFT JOIN members m ON el.member_id = m.id
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
            (m.first_name || ' ' || m.last_name) as member_name,
            mt.name as membership_type_name,
            el.card_id,
            el.entry_time,
            el.status,
            el.notes
        FROM entry_logs el
        LEFT JOIN members m ON el.member_id = m.id
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
