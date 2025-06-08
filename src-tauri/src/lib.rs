pub mod backup;
pub mod commands;
pub mod config;
pub mod db;
pub mod dto;
pub mod error;
pub mod models;
pub mod state;
pub mod utils;

pub use error::{AppError, Result};

pub use state::AppState;
pub fn library_setup_function() {
    tracing::debug!("Core application library initialized (placeholder)");
}
