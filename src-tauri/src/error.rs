use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Tauri API error: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("HTTP request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Backup failed: {0}")]
    BackupFailed(String),
    #[error("Restore failed: {0}")]
    RestoreFailed(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("API error: Status {status}, Message: {message}")]
    ApiError { status: u16, message: String },
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// Custom Result type alias
pub type Result<T> = std::result::Result<T, AppError>;
