use std::fmt;

use serde::{Deserialize, Serialize};
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

    #[error("Membership check failed: {0}")]
    MembershipCheckFailed(String),
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
    #[error("Translatable error: {0}")]
    Translatable(#[from] TranslatableError),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct ErrorMessage {
            message: String,
            error_code: Option<String>,
            params: Option<serde_json::Value>,
        }

        let error_message = ErrorMessage {
            message: self.to_string(),
            error_code: match self {
                AppError::Translatable(translatable_error) => {
                    Some(translatable_error.error_code.clone())
                }
                _ => None,
            },
            params: match self {
                AppError::Translatable(translatable_error) => translatable_error.params.clone(),
                _ => None,
            },
        };

        error_message.serialize(serializer)
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranslatableError {
    pub error_code: String,
    pub params: Option<serde_json::Value>,
    pub fallback_message: String,
}

impl TranslatableError {
    pub fn new(error_code: &str, fallback_message: &str) -> Self {
        Self {
            error_code: error_code.to_string(),
            params: None,
            fallback_message: fallback_message.to_string(),
        }
    }

    pub fn with_params(
        error_code: &str,
        params: serde_json::Value,
        fallback_message: &str,
    ) -> Self {
        Self {
            error_code: error_code.to_string(),
            params: Some(params),
            fallback_message: fallback_message.to_string(),
        }
    }
}

impl fmt::Display for TranslatableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.fallback_message)
    }
}
pub struct ErrorCodes;

impl ErrorCodes {
    pub const MEMBERSHIP_TYPE_NAME_EXISTS: &'static str = "error.membership_type_name_exists";
    pub const CARD_ALREADY_EXISTS: &'static str = "error.card_already_exists";
    pub const OVERLAPPING_MEMBERSHIP: &'static str = "error.overlapping_membership";
    pub const USERNAME_ALREADY_EXISTS: &'static str = "error.username_already_exists";
}

impl std::error::Error for TranslatableError {}

// Custom Result type alias
pub type Result<T> = std::result::Result<T, AppError>;
