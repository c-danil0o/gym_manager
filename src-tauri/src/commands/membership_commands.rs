use crate::dto::{MembershipInfo, MembershipPayload};
use crate::{
    error::{AppError, Result as AppResult},
    state::AppState,
};
use chrono::{NaiveDate, Utc};
use tauri::State;

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
        Ok("inactive".to_string())
    }
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
    if payload.membership_start_date.is_none() || payload.membership_start_date.is_none() {
        return Err(AppError::Validation(
            "Start date and end date are required for a membership.".to_string(),
        ));
    }
    if payload.membership_end_date <= payload.membership_start_date {
        return Err(AppError::Validation(
            "End date cannot be before start date.".to_string(),
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

    let final_status;
    if payload.membership_suspended.is_some() && payload.membership_suspended.unwrap() {
        final_status = "suspended".to_string();
    } else {
        final_status = determine_membership_status(
            &payload.membership_start_date,
            &payload.membership_end_date,
            payload.membership_remaining_visits,
        )
        .await?;
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

        // Check if member already has membership overlapping with the new one
        let overlapping_membership = sqlx::query_scalar!(
            r#"
          SELECT COUNT(*) FROM memberships
          WHERE member_id = ? AND is_deleted = FALSE
          AND (
          (start_date <= ? AND end_date >= ?)
          OR (start_date <= ? AND end_date IS NULL)
          )
          "#,
            payload.member_id,
            payload.membership_end_date,
            payload.membership_start_date,
            payload.membership_start_date,
        )
        .fetch_one(&state.db_pool)
        .await;
        match overlapping_membership {
            Ok(count) if count > 0 => {
                tracing::warn!(
                    "Member with ID {} already has an overlapping membership.",
                    payload.member_id
                );
                return Err(AppError::Validation(
                    "Member already has an overlapping membership.".to_string(),
                ));
            }
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Failed to check for overlapping memberships: {:?}", e);
                return Err(AppError::Sqlx(e));
            }
        }

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
        tracing::info!(
            "Successfully retrieved membership details for ID: {}",
            m.membership_id.unwrap_or(-1)
        );
        Ok(m)
    } else {
        Err(AppError::NotFound(format!(
            "Membership with ID {:?} not found after save.",
            final_membership_id
        )))
    }
}

#[tauri::command]
pub async fn delete_membership(id: i64, state: State<'_, AppState>) -> AppResult<()> {
    tracing::info!("Deleting membership with ID: {}", id);

    let now = Utc::now().naive_utc();
    let result = sqlx::query!(
        "UPDATE memberships SET is_deleted = TRUE, updated_at = ? WHERE id = ? AND is_deleted = FALSE",
        now,
        id
    )
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(_) => {
            tracing::info!("Successfully deleted membership with ID: {}", id);
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to delete membership with ID {}: {:?}", id, e);
            return Err(AppError::Sqlx(e));
        }
    }
}
