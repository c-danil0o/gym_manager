use crate::{
    error::{AppError, Result as AppResult},
    models::{Member, MemberInfo, MemberWithMembership, MembershipInfo, MembershipType},
    state::AppState,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Deserialize)]
pub struct MemberPayload {
  id: Option<i64>, // Optional ID for updates, required for new members
    card_id: String,
    first_name: String,
    last_name: String,
    email: Option<String>,
    phone: Option<String>,
    date_of_birth: Option<NaiveDate>,
}

#[derive(Deserialize)]
pub struct MembershipPayload {
    pub member_id: i64, // Member ID to associate with the membership

    pub membership_id: Option<i64>,
    pub membership_type_id: Option<i64>,
    pub membership_start_date: Option<NaiveDate>,
    pub membership_end_date: Option<NaiveDate>,
    pub membership_remaining_visits: Option<i64>,
}

#[derive(Serialize)]
pub struct MemberResponse {
    success: bool,
    message: String,
}

#[derive(Deserialize, Debug)]
pub struct GetMembersPaginatedPayload {
    page: Option<i32>,            // 1-indexed page number
    page_size: Option<i32>,       // Number of items per page
    search_query: Option<String>, // Optional search query
}

#[derive(Deserialize, Debug)]
pub struct GetMemberByIdPayload {
    id: i64, // Member ID to fetch
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MemberWithMembershipUpdate {
    pub id: i64, // Member ID
    pub card_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,

    pub membership_type_id: Option<i64>,

    pub membership_id: Option<i64>,
    pub membership_start_date: Option<NaiveDate>,
    pub membership_end_date: Option<NaiveDate>,
    pub membership_remaining_visits: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedMembersResponse {
    pub members: Vec<MemberInfo>,
    pub total_items: i64,
    pub total_pages: i64,
    pub current_page: i32,
    pub page_size: i32,
}

const DEFAULT_PAGE: i32 = 1;
const DEFAULT_PAGE_SIZE: i32 = 20;

#[tauri::command]
pub async fn add_member(
    payload: MemberPayload,
    state: State<'_, AppState>,
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
        ROW_NUMBER() OVER(PARTITION BY ms_inner.member_id ORDER BY
            CASE ms_inner.status WHEN 'active' THEN 0 ELSE 1 END,
            ms_inner.start_date DESC
        ) as rn
    FROM memberships ms_inner
    WHERE ms_inner.is_deleted = FALSE
) ms ON m.id = ms.member_id AND ms.rn = 1
LEFT JOIN
    membership_types mt ON ms.membership_type_id = mt.id AND mt.is_deleted = FALSE
WHERE
    m.is_deleted = FALSE
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
              ROW_NUMBER() OVER(PARTITION BY ms_inner.member_id ORDER BY
                  CASE ms_inner.status WHEN 'active' THEN 0 ELSE 1 END,
                  ms_inner.start_date DESC
              ) as rn
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

async fn determine_membership_status(
    start_date: &Option<NaiveDate>,
    end_date: &Option<NaiveDate>,
    remaining_visits: Option<i64>,
) -> AppResult<String> {
    let now_date = Utc::now().date_naive();
    if remaining_visits.is_some() && remaining_visits.unwrap_or(0) <= 0 {
        return Ok("expired".to_string());
    }

    let start_date = match start_date {
        None => return Ok("inactive".to_string()),
        Some(s) => s,
    };

    if start_date > &now_date {
        Ok("pending".to_string())
    } else if let Some(ed) = end_date {
        if ed < &now_date {
            Ok("expired".to_string())
        } else {
            Ok("active".to_string())
        }
    } else {
        Ok("active".to_string())
    }
}

#[tauri::command]
pub async fn save_member_with_membership(
    payload: MemberWithMembershipUpdate,
    state: State<'_, AppState>,
) -> AppResult<MemberWithMembership> {
    // Return the full updated/created details
    tracing::info!(
        "Attempting to save member and membership data: Member ID {:?}, Membership Type ID {:?}",
        payload.id,
        payload.membership_type_id
    );

    // --- Payload Validation ---
    if payload.first_name.trim().is_empty() || payload.last_name.trim().is_empty() {
        return Err(AppError::Validation(
            "First and last name are required.".to_string(),
        ));
    }

    let mut tx = state.db_pool.begin().await?; // Start a transaction
    let now = Utc::now().naive_utc();
    let short_card_id = payload.card_id.chars().take(4).collect::<String>();

    // --- 1. Update Member ---
    tracing::info!("Updating existing member with ID: {}", payload.id);
    sqlx::query!(
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
        payload.id
    )
    .execute(&mut *tx)
    .await?;

    // --- 2. Handle Membership ---
    // Only proceed with membership operations if either membership_id or membership_type_id is present
    if payload.membership_id.is_some() || payload.membership_type_id.is_some() {
        // Determine status based on provided dates, or use status from payload if frontend calculates it
        let final_status = determine_membership_status(
            &payload.membership_start_date,
            &payload.membership_end_date,
            payload.membership_remaining_visits,
        )
        .await?;

        if let Some(existing_membership_id) = payload.membership_id {
            tracing::info!(
                "Updating existing membership with ID: {}",
                existing_membership_id
            );
            // Here, you might decide if membership_type_id can change for an existing membership.
            // Typically, changing type means new membership. So, we'd usually update other fields.
            // If membership_type_id *is* allowed to change, ensure you fetch the new type and recalculate.

            // If only type_id is changed for an existing membership, it's ambiguous without more rules.
            // Let's assume for now an update only modifies dates, status, visits of the *existing* type.
            if payload.membership_type_id.is_some()
                && payload.membership_type_id
                    != sqlx::query_scalar!(
                        "SELECT membership_type_id FROM memberships WHERE id = ?",
                        existing_membership_id
                    )
                    .fetch_one(&mut *tx)
                    .await
                    .ok()
            {
                // This is complex: changing the type of an existing membership record.
                // Might be better to end the old one and create a new one.
                // For now, let's assume we are only updating details of the membership instance, not its fundamental type.
                // If you allow type change, you MUST refetch the MembershipType here to get its duration/visits for recalculation.
                tracing::warn!("Changing membership_type_id for an existing membership record is complex. Ensure logic handles this or disallow.");
            }

            sqlx::query!(
                r#"
                UPDATE memberships SET
                    start_date = ?, end_date = ?, remaining_visits = ?, status = ?, updated_at = ?
                WHERE id = ? AND member_id = ? AND is_deleted = FALSE
                "#,
                payload.membership_start_date,
                payload.membership_end_date,
                payload.membership_remaining_visits,
                final_status,
                now,
                existing_membership_id,
                payload.id // Ensure it belongs to the correct member
            )
            .execute(&mut *tx)
            .await?;
            tracing::info!("Membership {} updated.", existing_membership_id);
        } else if let Some(type_id) = payload.membership_type_id {
            // --- Scenario: Create New Membership (from type_id) ---
            tracing::info!(
                "Creating new membership for member ID: {} from type ID: {}",
                payload.id,
                type_id
            );

            if payload.membership_start_date.is_none() {
                return Err(AppError::Validation(
                    "Start date is required for a new membership.".to_string(),
                ));
            }

            // Fetch the membership type template to get duration, default visits etc.
            let mem_type = sqlx::query_as!(
                MembershipType,
                "SELECT * FROM membership_types WHERE id = ? AND is_deleted = FALSE",
                type_id
            )
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Membership type with ID {} not found.", type_id))
            })?;

            // Calculate end_date if not provided and type has duration
            let mut end_date_naive: Option<NaiveDate> = None;
            if payload.membership_end_date.is_none()
                && mem_type.duration_days.is_some()
                && mem_type.duration_days.unwrap_or(0) > 0
            {
                end_date_naive = Some(
                    payload.membership_start_date.unwrap()
                        + chrono::Duration::days(mem_type.duration_days.unwrap()),
                );
            } else {
                end_date_naive = payload.membership_end_date;
            }

            // Initial remaining visits from type, or payload if provided (payload might override)
            let initial_remaining_visits =
                payload.membership_remaining_visits.or(mem_type.visit_limit);

            // Optional: Deactivate other active memberships for this member if only one should be active
            // sqlx::query!("UPDATE memberships SET status = 'inactive', updated_at = ? WHERE member_id = ? AND status = 'active' AND is_deleted = FALSE", now, member_id)
            //     .execute(&mut *tx).await?;

            sqlx::query!(
                r#"
                INSERT INTO memberships (member_id, membership_type_id, start_date, end_date, remaining_visits, status, purchase_date, updated_at, created_at, is_deleted)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, FALSE)
                "#,
                payload.id,
                type_id,
                payload.membership_start_date,
                end_date_naive,
                initial_remaining_visits,
                final_status, // Calculated status
                now, // Purchase date
                now, // updated_at
                now  // created_at
            )
            .execute(&mut *tx)
            .await?;
            tracing::info!("New membership created for member {}.", payload.id);
        }
        // Else: No membership_id and no membership_type_id, so no membership action.
    } else {
        tracing::info!("No membership data provided in payload, skipping membership operations for member ID: {}", payload.id);
    }

    // --- 3. Commit Transaction ---
    tx.commit().await?;
    tracing::info!(
        "Successfully saved member and membership data for member ID: {}.",
        payload.id
    );

    // --- 4. Fetch and Return the Updated/Created State ---
    // Call your existing get command to return the full, fresh data
    let final_data =
        get_member_by_id_with_membership(GetMemberByIdPayload { id: payload.id }, state).await?; // Re-fetch

    // The previous command returns AppResult<Option<MemberWithMembership>>
    // This command should ideally return AppResult<MemberWithMembership> if save implies existence
    final_data.ok_or_else(|| {
        AppError::NotFound(format!(
            "Failed to retrieve saved member with ID {} after save.",
            payload.id
        ))
    })
}

#[tauri::command]
pub async fn get_all_memberships_for_member(
    id: i64,
    state: State<'_, AppState>,
) -> AppResult<Vec<MembershipInfo>> {
    tracing::info!("Fetching all memberships for member with ID: {}", id);

    let memberships = sqlx::query_as!(
        MembershipInfo,
        r#"
        SELECT
            m.id as member_id,
            m.first_name as member_first_name,
            m.last_name as member_last_name,
            mt.id as membership_type_id,
            mt.name as membership_type_name,
            ms.id as membership_id,
            ms.start_date as membership_start_date,
            ms.end_date as membership_end_date,
            ms.status as membership_status,
            ms.remaining_visits as membership_remaining_visits,
            ms.purchase_date as membership_purchase_date
        FROM
            members m
        JOIN memberships ms ON m.id = ms.member_id AND ms.is_deleted = FALSE
        LEFT JOIN membership_types mt ON ms.membership_type_id = mt.id AND mt.is_deleted = FALSE
        WHERE
            m.is_deleted = FALSE AND m.id = ?
        "#,
        id
    )
    .fetch_all(&state.db_pool)
    .await?;

    if memberships.is_empty() {
        tracing::warn!("No memberships found for member with ID: {}", id);
    } else {
        tracing::info!(
            "Found {} memberships for member with ID: {}",
            memberships.len(),
            id
        );
    }

    Ok(memberships)
}

#[tauri::command]
pub async fn get_membership_by_id(
    id: i64,
    state: State<'_, AppState>,
) -> AppResult<Option<MembershipInfo>> {
    tracing::info!("Fetching membership by ID: {}", id);

    let membership = sqlx::query_as!(
        MembershipInfo,
        r#"
        SELECT
            m.id as member_id,
            m.first_name as member_first_name,
            m.last_name as member_last_name,
            mt.id as membership_type_id,
            mt.name as membership_type_name,
            ms.id as membership_id,
            ms.start_date as membership_start_date,
            ms.end_date as membership_end_date,
            ms.status as membership_status,
            ms.remaining_visits as membership_remaining_visits,
            ms.purchase_date as membership_purchase_date
        FROM
            memberships ms
        JOIN members m ON ms.member_id = m.id AND m.is_deleted = FALSE
        LEFT JOIN membership_types mt ON ms.membership_type_id = mt.id AND mt.is_deleted = FALSE
        WHERE
            ms.is_deleted = FALSE AND ms.id = ?
        "#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some(m) = &membership {
        tracing::info!("Found membership for member ID: {}", m.member_id);
    } else {
        tracing::warn!("No membership found with ID: {}", id);
    }

    Ok(membership)
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

    if payload.first_name.trim().is_empty()
        || payload.last_name.trim().is_empty()
    {
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
            get_member_by_id(GetMemberByIdPayload { id: payload.id.unwrap() }, state).await?
                .ok_or_else(|| AppError::NotFound(format!("Member with ID {} not found after update.", payload.id.unwrap())))
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

#[tauri::command]
pub async fn save_membership(
    payload: MembershipPayload,
    state: State<'_, AppState>,
) -> AppResult<MembershipInfo> {
    tracing::info!(
        "Saving membership for member ID: {}, membership ID: {:?}",
        payload.member_id,
        payload.membership_id
    );

    // Validate member exists
    let member_exists = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM members WHERE id = ? AND is_deleted = FALSE",
        payload.member_id
    )
    .fetch_one(&state.db_pool)
    .await?;

    if member_exists == 0 {
        return Err(AppError::NotFound(format!(
            "Member with ID {} not found.",
            payload.member_id
        )));
    }

    let final_status = determine_membership_status(
        &payload.membership_start_date,
        &payload.membership_end_date,
        payload.membership_remaining_visits,
    )
    .await?;
    if payload.membership_start_date.is_none() || payload.membership_start_date.is_none() {
        return Err(AppError::Validation(
            "Start date and end date are required for a membership.".to_string(),
        ));
    }
    if payload.membership_remaining_visits.is_none() {
        return Err(AppError::Validation(
            "Remaining visits must be specified for a membership.".to_string(),
        ));
    }
    if payload.membership_type_id.is_none() {
        return Err(AppError::Validation(
            "Membership_type_id must be provided.".to_string(),
        ));
    }
    let mut final_membership_id = payload.membership_id;

    // If membership_id is provided, update existing membership
    if let Some(membership_id) = payload.membership_id {
        tracing::info!("Updating existing membership with ID: {}", membership_id);
        let now = Utc::now().naive_utc();

        sqlx::query!(
            r#"
            UPDATE memberships SET
                start_date = ?, end_date = ?, remaining_visits = ?,
                status = ?, updated_at = ?, membership_type_id = ?
            WHERE id = ? AND member_id = ? AND is_deleted = FALSE
            "#,
            payload.membership_start_date,
            payload.membership_end_date,
            payload.membership_remaining_visits,
            final_status,
            now,
            payload.membership_type_id,
            membership_id,
            payload.member_id
        )
        .execute(&state.db_pool)
        .await?;

        tracing::info!("Successfully updated membership with ID: {}", membership_id);
    } else if let Some(membership_type_id) = payload.membership_type_id {
        // Create new membership from type
        tracing::info!(
            "Creating new membership for member ID: {} from type ID: {}",
            payload.member_id,
            membership_type_id
        );

        let now = Utc::now().naive_utc();
        let insert_result = sqlx::query!(
            r#"
            INSERT INTO memberships (member_id, membership_type_id, start_date, end_date, remaining_visits, status, purchase_date, updated_at, created_at, is_deleted)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, FALSE)
            "#,
            payload.member_id,
            membership_type_id,
            payload.membership_start_date,
            payload.membership_end_date,
            payload.membership_remaining_visits,
            final_status,
            now, // purchase_date
            now, // updated_at
            now  // created_at
        )
        .execute(&state.db_pool)
        .await;

        match insert_result {
            Ok(result) => {
                tracing::info!("Successfully inserted new membership.");
                final_membership_id = Some(result.last_insert_rowid());
            }
            Err(e) => {
                tracing::error!("Failed to insert new membership: {:?}", e);
                return Err(AppError::Sqlx(e));
            }
        }
    }

  let membership = get_membership_by_id(final_membership_id.unwrap(), state).await?;
    if let Some(m) = membership {
        tracing::info!("Successfully retrieved membership details for ID: {}", m.membership_id.unwrap_or(-1));
        Ok(m)
    } else {
        Err(AppError::NotFound(format!(
            "Membership with ID {:?} not found after save.",
            final_membership_id
        )))
    }

}
