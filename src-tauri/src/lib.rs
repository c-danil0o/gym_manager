pub mod db;
pub mod error;
pub mod utils;
pub mod models;
pub mod state;
pub mod config;
pub mod backup;
pub mod commands;

pub use error::{AppError, Result};

pub use state::AppState;
pub fn library_setup_function() {
    tracing::debug!("Core application library initialized (placeholder)");
}
