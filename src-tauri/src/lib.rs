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

// Optional: Re-export configuration loading/saving functions if main.rs needs them directly
// pub use config::{load_settings, save_settings};

// Optional: Re-export core models if frequently used directly in main.rs setup
// pub use models::{AppSettings, Member};

// --- Library-Level Functions (Optional) ---
// You might have high-level functions here that orchestrate multiple modules,
// but often, the setup and orchestration happen in main.rs using the modules directly.

// Example (less common for this setup):
// pub async fn initialize_app_core(app_handle: &tauri::AppHandle) -> Result<(sqlx::SqlitePool, config::AppSettings)> {
//     let settings = config::load_settings(app_handle).await?;
//     let pool = db::init_db(app_handle).await?;
//     // maybe do other core setup steps...
//     Ok((pool, settings))
// }

// --- Library Setup (If needed beyond module declarations) ---
// Usually empty if modules handle their own specifics.
pub fn library_setup_function() {
    // Potentially initialize things specific to the library itself,
    // though often not needed for this kind of application structure.
    tracing::debug!("Core application library initialized (placeholder)");
}
