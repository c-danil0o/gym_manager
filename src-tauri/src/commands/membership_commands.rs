use crate::{
    error::{AppError, Result as AppResult},
    models::MembershipType,
    state::AppState,
    utils,
};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Deserialize)]
pub struct NewMembershipTypePayload {
    name: String,
    duration_days: Option<i64>,
    visit_limit: Option<i64>,
    enter_by: Option<i64>,
    price: f64,
    description: Option<String>,
}

#[derive(Serialize)]
pub struct MembershipResponse {
    success: bool,
    message: String,
}

#[tauri::command]
pub async fn add_membership_type(
    payload: NewMembershipTypePayload,
    state: State<'_, AppState>,
) -> AppResult<MembershipType> {
    tracing::info!("Creating new membership type: {}", &payload.name);

    if payload.name.trim().is_empty() {
            tracing::warn!("Validation failed: Membership type name cannot be empty.");
            return Err(AppError::Validation("Membership type name cannot be empty.".to_string()));
        }
        if payload.price < 0.0 {
            tracing::warn!("Validation failed: Price cannot be negative.");
            return Err(AppError::Validation("Price cannot be negative.".to_string()));
        }
        if let Some(duration) = payload.duration_days {
            if duration <= 0 {
                return Err(AppError::Validation("Duration days must be positive if provided.".to_string()));
            }
        }
        if let Some(limit) = payload.visit_limit {
            if limit <= 0 {
                return Err(AppError::Validation("Visit limit must be positive if provided.".to_string()));
            }
        }

        let now = chrono::Utc::now().naive_utc();

        let result = sqlx::query!(
            r#"
            INSERT INTO membership_types (name, duration_days, visit_limit, enter_by, price, description, created_at, updated_at, is_deleted)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, FALSE)
            "#,
            payload.name,
            payload.duration_days,
            payload.visit_limit,
            payload.enter_by,
            payload.price,
            payload.description,
            now
        )
        .execute(&state.db_pool)
        .await;

        match result {
            Ok(query_result) => {
                let last_insert_id = query_result.last_insert_rowid();
                tracing::info!(
                    "Successfully inserted new membership type '{}' with id {}.",
                    payload.name,
                    last_insert_id
                );

                // Fetch the newly created membership type to return it
                let new_type = sqlx::query_as!(
                    MembershipType,
                    r#"
                    SELECT id, name, duration_days, visit_limit, price, enter_by, description, created_at, updated_at, is_deleted
                    FROM membership_types
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
                    "Failed to create membership type: Name '{}' already exists. Error: {:?}",
                    payload.name,
                    db_err
                );
                Err(AppError::Validation(format!(
                    "A membership type with the name '{}' already exists.",
                    payload.name
                )))
            }
            Err(e) => {
                tracing::error!("Failed to insert new membership type into database: {:?}", e);
                Err(AppError::Sqlx(e)) // Convert general SQLx errors
            }
        }
}

#[tauri::command]
pub async fn get_all_membership_types(
    state: State<'_, AppState>,
) -> AppResult<Vec<MembershipType>> {
    tracing::info!("Fetching all membership types.");
    let types = sqlx::query_as!(
        MembershipType,
        r#"
        SELECT id as 'id!', name, duration_days, visit_limit, price, enter_by, description, created_at, updated_at, is_deleted
        FROM membership_types
        WHERE is_deleted = FALSE
        ORDER BY name ASC
        "#
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(types)
}

#[tauri::command]
pub async fn delete_membership_type(id: i64, state: State<'_, AppState>) -> AppResult<()> {
    tracing::info!("Attempting to (soft) delete membership type with id: {}", id);

    let now = chrono::Utc::now().naive_utc();
    let result = sqlx::query!(
        "UPDATE membership_types SET is_deleted = TRUE, updated_at = ? WHERE id = ?",
        now,
        id
    )
    .execute(&state.db_pool)
    .await?;

    if result.rows_affected() == 0 {
        tracing::warn!("No membership type found with id {} to delete.", id);
        return Err(AppError::NotFound(format!(
            "Membership type with id {} not found.",
            id
        )));
    }
    // will need to delete all memberships with this type

    tracing::info!("Successfully soft-deleted membership type with id: {}", id);
    Ok(())
}
