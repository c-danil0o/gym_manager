use crate::dto::{
    GetMemberByIdPayload, GetMembersPaginatedPayload, InviteMemberPayload, MemberInfo,
    MemberPayload, MemberWithMembership, PaginatedResponse,
};
use crate::error::{ErrorCodes, TranslatableError};
use crate::sync::trigger_instant_sync;
use crate::utils;
use crate::{
    error::{AppError, Result as AppResult},
    models::Member,
    state::AppState,
};
use supabase_auth::error::Error;
use supabase_auth::models::{AuthClient, User};
use tauri::State;

const DEFAULT_PAGE: i32 = 1;
const DEFAULT_PAGE_SIZE: i32 = 20;
const MY_REDIRECT_URL_CONSTANT: &str = "http://localhost:5173";

#[tauri::command]
pub async fn add_member(
    payload: MemberPayload,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> AppResult<Member> {
    tracing::info!(
        "Creating new member: {} {}",
        &payload.first_name,
        &payload.last_name
    );

    if payload.card_id.trim().is_empty() {
        tracing::warn!("Validation failed: Card id must be specified.");
        return Err(AppError::Validation(
            "Card id must not be empty!".to_string(),
        ));
    }

    if payload.first_name.trim().is_empty() || payload.last_name.trim().is_empty() {
        tracing::warn!("Validation failed: First and last name are required!.");
        return Err(AppError::Validation(
            "Required first and last name!".to_string(),
        ));
    }

    let now = chrono::Utc::now().naive_utc();
    let short_card_id = payload.card_id.chars().take(4).collect::<String>();

    let result = sqlx::query!(
            r#"
            INSERT INTO members (card_id, short_card_id, first_name, last_name, email, phone, date_of_birth, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
            "#,
            payload.card_id,
            short_card_id,
            payload.first_name,
            payload.last_name,
            payload.email,
            payload.phone,
            payload.date_of_birth,
            now
        )
        .execute(&state.db_pool)
        .await;

    match result {
        Ok(query_result) => {
            let last_insert_id = query_result.last_insert_rowid();
            tracing::info!(
                "Successfully inserted new member '{}' with id {}.",
                payload.first_name,
                last_insert_id
            );
            trigger_instant_sync(&app_handle);

            let new_type = sqlx::query_as!(
                    Member,
                    r#"
                    SELECT id, card_id, short_card_id, first_name, last_name, email, phone, date_of_birth, created_at, updated_at, is_deleted, auth_user_id
                    FROM members
                    WHERE id = ?
                    "#,
                    last_insert_id
                )
                .fetch_one(&state.db_pool)
                .await?;

            Ok(new_type)
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            tracing::warn!(
                "Failed to create member: card_id {} exists! Error: {:?}",
                payload.card_id,
                db_err
            );
            Err(AppError::Translatable(TranslatableError::with_params(
                ErrorCodes::CARD_ALREADY_EXISTS,
                serde_json::json!({"card_id": payload.card_id}),
                "failed to create member: card_id already exists!",
            )))
        }
        Err(e) => {
            tracing::error!("Failed to insert new member into database: {:?}", e);
            Err(AppError::Sqlx(e)) // Convert general SQLx errors
        }
    }
}
const RELEVANT_MEMBERSHIP_CTE: &str = r#"
WITH latest_memberships AS (
    SELECT
        ms_inner.id,
        ms_inner.member_id,
        ms_inner.membership_type_id,
        ms_inner.status,
        ms_inner.start_date,
        ROW_NUMBER() OVER (
            PARTITION BY ms_inner.member_id
            ORDER BY
                CASE ms_inner.status WHEN 'active' THEN 0 ELSE 1 END ASC,
                CASE ms_inner.status WHEN 'pending' THEN 0 ELSE 1 END ASC,
                CASE WHEN ms_inner.status = 'pending' THEN ms_inner.start_date ELSE NULL END ASC,
                CASE WHEN ms_inner.status = 'active' THEN ms_inner.start_date ELSE NULL END DESC,
                ms_inner.start_date DESC
        ) AS rn
    FROM memberships ms_inner
    WHERE (ms_inner.is_deleted IS NULL OR ms_inner.is_deleted = FALSE)
)"#;

const MEMBER_DATA_SELECT_SQL: &str = r#"
SELECT
    m.id, m.card_id, m.first_name, m.last_name, m.email, m.phone, m.created_at as member_created_at,
    m.first_name || ' ' || m.last_name as name,
    ms.id as membership_id,
    mt.name as membership_type_name,
    ms.status as membership_status
FROM
    members m
LEFT JOIN latest_memberships ms
    ON m.id = ms.member_id AND ms.rn = 1
LEFT JOIN
    membership_types mt ON ms.membership_type_id = mt.id AND (mt.is_deleted IS NULL OR mt.is_deleted = FALSE)
WHERE 1 = 1
"#;

const COUNT_MEMBERS_QUERY: &str = r#"
SELECT COUNT(DISTINCT m.id)
FROM members m
LEFT JOIN latest_memberships ms ON m.id = ms.member_id AND ms.rn = 1
LEFT JOIN membership_types mt ON ms.membership_type_id = mt.id
    AND (mt.is_deleted IS NULL OR mt.is_deleted = FALSE)
WHERE 1 = 1
"#;

#[tauri::command]
pub async fn get_members_with_memberships_paginated(
    payload: GetMembersPaginatedPayload,
    state: State<'_, AppState>,
) -> AppResult<PaginatedResponse<MemberInfo>> {
    let current_page = payload.page.unwrap_or(DEFAULT_PAGE).max(1);
    let page_size = payload.per_page.unwrap_or(DEFAULT_PAGE_SIZE).max(1);
    let offset = (current_page - 1) * page_size;

    let search_term = payload
        .search_string
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty());

    if let Some(order_by) = &payload.order_by {
        if !["name", "card_id", "membership_status"].contains(&order_by.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid order_by field: {}. Allowed values are: name, card_id, membership_status.",
                order_by
            )));
        }
    }
    let order_direction = match &payload.order_direction {
        Some(dir) => {
            if !["asc", "desc"].contains(&dir.as_str()) {
                return Err(AppError::Validation(format!(
                    "Invalid order_direction: {}. Allowed values are: asc, desc.",
                    dir
                )));
            }
            dir.to_uppercase()
        }
        None => "ASC".to_string(),
    };

    let members_data: Vec<MemberInfo>;
    let total_items: i64;
    let order_by_query = match payload.order_by.as_deref() {
        Some("name") => format!(
            "ORDER BY m.first_name {}, m.last_name {}",
            order_direction, order_direction
        ),
        Some("card_id") => format!("ORDER BY m.card_id {}", order_direction),
        Some("membership_status") => format!("ORDER BY ms.status {}", order_direction),
        _ => "ORDER BY m.last_name ASC, m.first_name ASC".to_string(),
    };
    let mut filter_query = "TRUE".to_string();

    if let Some(filter_fields) = &payload.filter_fields {
        let mut filter_conditions = Vec::new();
        for field in filter_fields {
            match field.field.as_str() {
                "membership_type_name" => {
                    filter_conditions.push(format!(
                        "COALESCE(mt.id, 0) IN ({})",
                        field
                            .value
                            .split(',')
                            .map(|s| s.trim())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                "membership_status" => {
                    filter_conditions.push(format!(
                        "COALESCE(ms.status, '') IN ('{}')",
                        field
                            .value
                            .split(',')
                            .map(|s| s.trim())
                            .collect::<Vec<_>>()
                            .join("', '")
                    ));
                }
                _ => {
                    return Err(AppError::Validation(format!(
                        "Invalid filter field: {}. Allowed values are: membership_type_name, membership_status.",
                        field.field
                    )));
                }
            }
        }

        if !filter_conditions.is_empty() {
            filter_query = filter_conditions.join(" AND ");
        }
    }
    let search_query;

    if let Some(term) = &search_term {
        let like_pattern = format!("'%{}%'", term); // Prepare for LIKE

        search_query = format!("(LOWER(m.first_name) LIKE {val} OR LOWER(m.last_name) LIKE {val} OR LOWER(m.first_name || ' ' || m.last_name) LIKE {val}) OR m.card_id LIKE {val}", val = like_pattern);
    } else {
        search_query = "TRUE".to_string();
    }
    let pagination_query = "LIMIT $1 OFFSET $2".to_string();

    let query_string = format!(
        "{} {} AND {} AND {} {} {}",
        RELEVANT_MEMBERSHIP_CTE,
        MEMBER_DATA_SELECT_SQL,
        filter_query,
        search_query,
        order_by_query,
        pagination_query
    );

    tracing::info!(
        "Executing query: {} with page_size: {}, offset: {}",
        query_string,
        page_size,
        offset
    );

    members_data = sqlx::query_as(&query_string) // Using query_as with runtime string
        .bind(page_size as i64) // SQLx expects i64 for LIMIT/OFFSET
        .bind(offset as i64)
        .fetch_all(&state.db_pool)
        .await?;

    let count_query_string = format!(
        "{} {} AND {} AND {}",
        RELEVANT_MEMBERSHIP_CTE, COUNT_MEMBERS_QUERY, filter_query, search_query
    );
    total_items = sqlx::query_scalar(&count_query_string)
        .fetch_one(&state.db_pool)
        .await?;

    let total_pages = if page_size > 0 {
        (total_items as f64 / page_size as f64).ceil() as i64
    } else {
        0
    };

    Ok(PaginatedResponse {
        data: members_data,
        total: total_items,
        total_pages: total_pages,
        page: current_page,
        per_page: page_size,
    })
}

#[tauri::command]
pub async fn get_member_by_id_with_membership(
    payload: GetMemberByIdPayload,
    state: State<'_, AppState>,
) -> AppResult<Option<MemberWithMembership>> {
    let member_id = payload.id;

    tracing::info!(
        "Fetching member with membership details for member_id: {}",
        member_id
    );

    utils::check_membership_statuses(&state).await?;
    let query_result = sqlx::query_as!(
      MemberWithMembership,
      r#"
      SELECT
          m.id as id, m.card_id, m.short_card_id, m.first_name, m.last_name, m.email, m.date_of_birth, m.phone, m.created_at as member_created_at,
          ms.id as membership_id,
          ms.start_date as membership_start_date,
          ms.end_date as membership_end_date,
          ms.remaining_visits as membership_remaining_visits,
          ms.purchase_date as membership_purchase_date,
          ms.status as membership_status,
          mt.name as membership_type_name,
          mt.id as membership_type_id,
          mt.duration_days as membership_type_duration_days,
          mt.visit_limit as membership_type_visit_limit,
          mt.enter_by as membership_type_enter_by,
          mt.price as membership_type_price
      FROM
          members m
      LEFT JOIN (
          SELECT
            ms_inner.*,
            ROW_NUMBER() OVER (
                PARTITION BY ms_inner.member_id
                ORDER BY
                    -- 1. Prioritize 'active' status first
                    CASE ms_inner.status WHEN 'active' THEN 0 ELSE 1 END ASC,
                    -- 2. If not active, prioritize 'pending' status next
                    CASE ms_inner.status WHEN 'pending' THEN 0 ELSE 1 END ASC,
                    -- 3. For 'pending' memberships, pick the one with the closest future start_date
                    CASE WHEN ms_inner.status = 'pending' THEN ms_inner.start_date ELSE NULL END ASC, -- NULLS LAST for pending means future dates come first
                    -- 4. For 'active' memberships, pick the most recent start_date
                    CASE WHEN ms_inner.status = 'active' THEN ms_inner.start_date ELSE NULL END DESC,
                    -- 5. For all other statuses (expired, inactive, suspended), pick the most recent start_date
                    ms_inner.start_date DESC
            ) AS rn
          FROM memberships ms_inner
          WHERE ms_inner.is_deleted = FALSE
      ) ms ON m.id = ms.member_id AND ms.rn = 1
      LEFT JOIN
          membership_types mt ON ms.membership_type_id = mt.id AND mt.is_deleted = FALSE
      WHERE
          m.id = ?
      "#,
      member_id
  ).fetch_optional(&state.db_pool).await;

    match query_result {
        Ok(Some(data)) => {
            tracing::info!("Successfully fetched member data for ID: {}", member_id);
            Ok(Some(data))
        }
        Ok(None) => {
            tracing::warn!("Member with ID {} not found (fetch_optional).", member_id);
            Ok(None)
        }
        Err(e) => {
            tracing::error!(
                "Database error while fetching member with ID {}: {:?}",
                member_id,
                e
            );
            Err(AppError::Sqlx(e))
        }
    }
}

#[tauri::command]
pub async fn get_member_by_id(
    payload: GetMemberByIdPayload,
    state: State<'_, AppState>,
) -> AppResult<Option<Member>> {
    let member_id = payload.id;

    tracing::info!("Fetching member by ID: {}", member_id);
    let member = sqlx::query_as!(
        Member,
        r#"
        SELECT id, card_id, short_card_id, first_name, last_name, email, phone, date_of_birth, created_at, updated_at, is_deleted, auth_user_id
        FROM members
        WHERE id = ? AND is_deleted = FALSE
        "#,
        member_id
    )
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some(m) = &member {
        tracing::info!("Found member: {} {}", m.first_name, m.last_name);
    } else {
        tracing::warn!("No member found with ID: {}", member_id);
    }

    Ok(member)
}
#[tauri::command]
pub async fn delete_member(
    id: i64,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> AppResult<()> {
    tracing::info!("Attempting to delete member with id: {}", id);

    let result = sqlx::query!("DELETE FROM members WHERE id = ?", id)
        .execute(&state.db_pool)
        .await?;

    if result.rows_affected() == 0 {
        tracing::warn!("No member found with id {} to delete.", id);
        return Err(AppError::NotFound(format!(
            "Member with id {} not found.",
            id
        )));
    }

    trigger_instant_sync(&app_handle);

    tracing::info!("Successfully member with id: {}", id);
    Ok(())
}

#[tauri::command]
pub async fn update_member(
    payload: MemberPayload,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> AppResult<Member> {
    tracing::info!(
        "Updating member: {} {}",
        &payload.first_name,
        &payload.last_name
    );

    if payload.id.is_none() {
        return Err(AppError::Validation(
            "Member ID is required for updates.".to_string(),
        ));
    }
    let member_id = payload.id.unwrap();

    if payload.first_name.trim().is_empty() || payload.last_name.trim().is_empty() {
        return Err(AppError::Validation(
            "First and last name are required.".to_string(),
        ));
    }

    if payload.card_id.trim().is_empty() {
        return Err(AppError::Validation(
            "Card id must not be empty!".to_string(),
        ));
    }

    let now = chrono::Utc::now().naive_utc();
    let short_card_id = payload.card_id.chars().take(4).collect::<String>();

    let result = sqlx::query!(
        r#"
        UPDATE members SET
            card_id = ?, short_card_id = ?, first_name = ?, last_name = ?,
            email = ?, phone = ?, date_of_birth = ?, updated_at = ?
        WHERE id = ? AND is_deleted = FALSE
        "#,
        payload.card_id,
        short_card_id,
        payload.first_name,
        payload.last_name,
        payload.email,
        payload.phone,
        payload.date_of_birth,
        now,
        member_id
    )
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(_) => {
            tracing::info!("Successfully updated member with ID: {}", member_id);
            trigger_instant_sync(&app_handle);
            get_member_by_id(
                GetMemberByIdPayload {
                    id: payload.id.unwrap(),
                },
                state,
            )
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Member with ID {} not found after update.",
                    payload.id.unwrap()
                ))
            })
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            tracing::warn!(
                "Failed to update member: card_id {} or email already exists! Error: {:?}",
                payload.card_id,
                db_err
            );
            Err(AppError::Translatable(TranslatableError::with_params(
                ErrorCodes::CARD_ALREADY_EXISTS,
                serde_json::json!({"card_id": payload.card_id}),
                "failed to create member: card_id already exists!",
            )))
        }
        Err(e) => {
            tracing::error!("Failed to update member in database: {:?}", e);
            Err(AppError::Sqlx(e)) // Convert general SQLx errors
        }
    }
}

#[tauri::command]
pub async fn invite_member_to_app(
    payload: InviteMemberPayload,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> AppResult<()> {
    let pool = &state.db_pool;

    // Get member details
    let member =
        sqlx::query_as::<_, Member>("SELECT * FROM members WHERE id = ? AND is_deleted = FALSE")
            .bind(payload.member_id)
            .fetch_one(pool)
            .await?;

    let email = member
        .email
        .ok_or_else(|| AppError::Validation("Member must have email to be invited".to_string()))?;

    // Get Supabase settings
    let settings = state.settings.read().await;
    let supabase_url = settings
        .supabase_url
        .as_ref()
        .ok_or_else(|| AppError::Config("Supabase URL not configured".to_string()))?;
    let supabase_key = settings
        .supabase_key
        .as_ref() // Service role key needed
        .ok_or_else(|| AppError::Config("Supabase service role key not configured".to_string()))?;
    let supabase_jwt = settings
        .supabase_jwt_secret
        .as_ref() // Service role key needed
        .ok_or_else(|| AppError::Config("Supabase jwt secret not configured".to_string()))?;

    // Create Supabase auth client
    let auth_client = AuthClient::new(supabase_url, supabase_key, supabase_jwt);

    let mut redirect_data = serde_json::Map::new();
    redirect_data.insert(
        "redirectTo".to_string(),
        serde_json::Value::String(MY_REDIRECT_URL_CONSTANT.to_string()),
    );
    let redirect_data_value = serde_json::Value::Object(redirect_data);

    // Send invitation via Supabase Auth
    let invitation_result: Result<User, Error> = auth_client
        .invite_user_by_email(&email, Some(redirect_data_value), auth_client.api_key())
        .await;
    if let Err(e) = &invitation_result {
        tracing::error!("Failed to send invitation email via Supabase: {:?}", e);
        return Err(AppError::ExternalService(format!(
            "Failed to send invitation email: {:?}",
            e
        )));
    }
    let user: User = invitation_result.unwrap();
    let user_id = user.id.to_string();

    // Update member record to track invitation
    sqlx::query!(
        "UPDATE members SET
         auth_user_id = ?
         WHERE id = ?",
        user_id,
        member.id
    )
    .execute(pool)
    .await?;

    // Trigger sync to update Supabase
    trigger_instant_sync(&app_handle);

    tracing::info!("Invited member {} ({}) to mobile app", member.id, email);
    Ok(())
}
