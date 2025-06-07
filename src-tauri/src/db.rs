use crate::error::{AppError, Result};
use crate::utils;
use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions, SqlitePool};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn init_db(app_handle: &AppHandle) -> Result<SqlitePool> {
    let db_path = get_database_path(app_handle)?;

    let db_path = db_path.join("gym_data.sqlite");

    // Ensure the directory exists before connecting
    if let Some(parent_dir) = db_path.parent() {
        std::fs::create_dir_all(parent_dir)?;
    }

    // Check if database file exists, if not, create an empty file
    if !db_path.exists() {
        tracing::info!("Database file does not exist, creating it...");
        // Create an empty file to ensure SQLite can open it
        std::fs::File::create(&db_path)
            .map_err(|e| AppError::Config(format!("Failed to create database file: {}", e)))?;
        tracing::info!("Created empty database file at: {:?}", db_path);
    }

    // Use file:// prefix for SQLite and ensure absolute path
    let db_url = format!(
        "sqlite://{}",
        db_path
            .to_str()
            .ok_or_else(|| { AppError::Config("Invalid database path string".to_string()) })?
    );

    tracing::info!("Connecting to database at: {}", db_url);

    // Set pragmas for better SQLite performance and reliability
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&db_url)
        .await
        .map_err(|e| AppError::Config(format!("Failed to connect to database: {}", e)))?;

    // Execute pragmas for better SQLite behavior
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await
        .map_err(|e| AppError::Config(format!("Failed to set journal mode: {}", e)))?;
    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(&pool)
        .await
        .map_err(|e| AppError::Config(format!("Failed to set synchronous mode: {}", e)))?;
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .map_err(|e| AppError::Config(format!("Failed to enable foreign keys: {}", e)))?;

    // Run migrations
    tracing::info!("Running database migrations...");
    MIGRATOR.run(&pool).await?;
    tracing::info!("Database migrations completed.");
    create_default_admin_user_if_not_exists(&pool).await?;

    Ok(pool)
}

pub fn get_database_path(app_handle: &AppHandle) -> Result<PathBuf> {
    let app_dir = app_handle.path().app_data_dir().map_err(|_| {
        AppError::Config("Could not determine application data directory".to_string())
    })?;

    // Create directory synchronously since we're in a sync context
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
    }

    // Log the actual path for debugging
    tracing::debug!("Database will be stored at: {:?}", app_dir);

    Ok(app_dir)
}

async fn create_default_admin_user_if_not_exists(pool: &SqlitePool) -> Result<()> {
    let default_username = "admin";
    let default_password = "admin";

    // Check if the admin user already exists
    let admin_exists: Option<bool> = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM users WHERE username = ?1 LIMIT 1) as "exists: bool""#,
        default_username
    )
    .fetch_one(pool)
    .await?;

    if admin_exists.unwrap_or(false) {
        tracing::info!(
            "Default admin user '{}' already exists. Skipping creation.",
            default_username
        );
        return Ok(());
    }

    tracing::info!("Creating default admin user '{}'...", default_username);

    let password_hash = utils::hash_password(default_password)?;

    let now = chrono::Utc::now().naive_utc();

    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        INSERT INTO users (username, password_hash, role, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
        default_username,
        password_hash,
        "admin",
        now,
        now
    )
    .execute(&mut *tx)
    .await?;

    // Commit the transaction
    tx.commit().await?;

    tracing::info!(
        "Default admin user '{}' created successfully.",
        default_username
    );
    Ok(())
}
