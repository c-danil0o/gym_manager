use crate::dto::{
    GetMemberByIdPayload, GetMembersPaginatedPayload, MemberInfo, MemberPayload,
    MemberWithMembership, PaginatedMembersResponse,
};
use crate::{
    error::{AppError, Result as AppResult},
    models::Member,
    state::AppState,
};
use tauri::State;

const DEFAULT_PAGE: i32 = 1;
const DEFAULT_PAGE_SIZE: i32 = 20;

#[tauri::command]
pub async fn add_member(payload: MemberPayload, state: State<'_, AppState>) -> AppResult<Member> {
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

            let new_type = sqlx::query_as!(
                    Member,
                    r#"
                    SELECT id, card_id, short_card_id, first_name, last_name, email, phone, date_of_birth, created_at, updated_at, is_deleted
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
                "Failed to create member: card_id {} or email alrady exists! Error: {:?}",
                payload.card_id,
                db_err
            );
            Err(AppError::Validation(format!(
                "A card_id or email already exists. Card: {}",
                payload.card_id
            )))
        }
        Err(e) => {
            tracing::error!("Failed to insert new member into database: {:?}", e);
            Err(AppError::Sqlx(e)) // Convert general SQLx errors
        }
    }
}

const MEMBER_DATA_SELECT_SQL: &str = r#"
SELECT
    m.id, m.card_id, m.first_name, m.last_name, m.email, m.phone, m.created_at as member_created_at,
    ms.id as membership_id,
    mt.name as membership_type_name,
    ms.status as membership_status
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
    FROM
        memberships ms_inner
    WHERE
        (ms_inner.is_deleted IS NULL OR ms_inner.is_deleted = FALSE)
) ms ON m.id = ms.member_id AND ms.rn = 1
LEFT JOIN
    membership_types mt ON ms.membership_type_id = mt.id AND (mt.is_deleted IS NULL OR mt.is_deleted = FALSE)
WHERE
    (m.is_deleted IS NULL OR m.is_deleted = FALSE)
"#;

#[tauri::command]
pub async fn get_members_with_memberships_paginated(
    payload: GetMembersPaginatedPayload,
    state: State<'_, AppState>,
) -> AppResult<PaginatedMembersResponse> {
    let current_page = payload.page.unwrap_or(DEFAULT_PAGE).max(1);
    let page_size = payload.page_size.unwrap_or(DEFAULT_PAGE_SIZE).max(1);
    let offset = (current_page - 1) * page_size;

    let search_term = payload
        .search_query
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty());

    let members_data: Vec<MemberInfo>;
    let total_items: i64;

    if let Some(term) = search_term {
        let like_pattern = format!("%{}%", term); // Prepare for LIKE

        // --- Fetch Data with Search ---
        let query_string = format!(
            "{} AND (LOWER(m.first_name) LIKE $1 OR LOWER(m.last_name) LIKE $1 OR LOWER(m.first_name || ' ' || m.last_name) LIKE $1) OR m.card_id LIKE $1 ORDER BY m.last_name ASC, m.first_name ASC LIMIT $2 OFFSET $3",
            MEMBER_DATA_SELECT_SQL
        );
        members_data = sqlx::query_as(&query_string) // Using query_as with runtime string
            .bind(&like_pattern)
            .bind(page_size as i64) // SQLx expects i64 for LIMIT/OFFSET
            .bind(offset as i64)
            .fetch_all(&state.db_pool)
            .await?;

        // --- Count Data with Search ---
        let count_query_string = format!(
            "SELECT COUNT(DISTINCT m.id) FROM members m WHERE m.is_deleted = FALSE AND (LOWER(m.first_name) LIKE $1 OR LOWER(m.last_name) LIKE $1 OR LOWER(m.first_name || ' ' || m.last_name) LIKE $1)"
        );
        total_items = sqlx::query_scalar(&count_query_string)
            .bind(&like_pattern)
            .fetch_one(&state.db_pool)
            .await?;
    } else {
        // --- Fetch Data without Search ---
        let query_string = format!(
            "{} ORDER BY m.last_name ASC, m.first_name ASC LIMIT $1 OFFSET $2",
            MEMBER_DATA_SELECT_SQL
        );
        members_data = sqlx::query_as(&query_string)
            .bind(page_size as i64)
            .bind(offset as i64)
            .fetch_all(&state.db_pool)
            .await?;

        // --- Count Data without Search ---
        total_items = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT m.id) FROM members m WHERE m.is_deleted = FALSE",
        )
        .fetch_one(&state.db_pool)
        .await?;
    }

    let total_pages = if page_size > 0 {
        (total_items as f64 / page_size as f64).ceil() as i64
    } else {
        0
    };

    Ok(PaginatedMembersResponse {
        members: members_data,
        total_items,
        total_pages,
        current_page,
        page_size,
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
          m.is_deleted = FALSE AND m.id = ?
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
        SELECT id, card_id, short_card_id, first_name, last_name, email, phone, date_of_birth, created_at, updated_at, is_deleted
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
pub async fn delete_member(id: i64, state: State<'_, AppState>) -> AppResult<()> {
    tracing::info!("Attempting to (soft) delete member with id: {}", id);

    let now = chrono::Utc::now().naive_utc();
    let result = sqlx::query!(
        "UPDATE members SET is_deleted = TRUE, updated_at = ? WHERE id = ?",
        now,
        id
    )
    .execute(&state.db_pool)
    .await?;

    if result.rows_affected() == 0 {
        tracing::warn!("No member found with id {} to delete.", id);
        return Err(AppError::NotFound(format!(
            "Member with id {} not found.",
            id
        )));
    }

    tracing::info!("Successfully soft-deleted member with id: {}", id);
    Ok(())
}

#[tauri::command]
pub async fn update_member(
    payload: MemberPayload,
    state: State<'_, AppState>,
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
            Err(AppError::Validation(format!(
                "A card_id or email already exists. Card: {}",
                payload.card_id
            )))
        }
        Err(e) => {
            tracing::error!("Failed to update member in database: {:?}", e);
            Err(AppError::Sqlx(e)) // Convert general SQLx errors
        }
    }
}
