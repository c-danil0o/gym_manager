// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use gym_manager_lib::{
    backup,
    config,
    db,
    AppState,
    commands,
};
use tauri::Manager;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter("gym_manager=debug,tauri=info,sqlx=debug")
        .with_writer(std::io::stdout)
        .init();
    println!("=== Application starting ===");

    // Initialize the Tauri application builder
    let builder = tauri::Builder::default();

    // --- Run Application Setup ---
    let final_builder = builder.setup(|app| {
        // --- App Handle ---
        let app_handle = app.handle().clone();

        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        let settings =
            tauri::async_runtime::block_on(config::load_settings(&app_handle)).map_err(Box::new)?;
        tracing::info!("Loaded settings: {:?}", settings);

        // Verify app data directory access
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data directory");
        tracing::info!("App data directory: {:?}", app_dir);

        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir).expect("Failed to create app data directory");
            tracing::info!("Created app data directory");
        }

        // Verify permissions by writing a test file
        let test_file_path = app_dir.join(".write_test");
        match std::fs::File::create(&test_file_path) {
            Ok(_) => {
                // Clean up test file
                let _ = std::fs::remove_file(test_file_path);
                tracing::info!("Directory is writable");
            }
            Err(e) => {
                tracing::error!("Directory is not writable: {}", e);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("App data directory is not writable: {}", e),
                )));
            }
        }

        // --- Initialize Database ---
        let pool = match rt.block_on(db::init_db(&app_handle)) {
            Ok(pool) => pool,
            Err(e) => {
                tracing::error!("Database initialization failed: {:?}", e);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Database initialization failed: {:?}", e),
                )));
            }
        };
        tracing::info!("Database pool initialized and migrations run.");

        // --- Create and Manage State ---
        let app_state = AppState::new(pool.clone());
        app.manage(app_state); // Register the state with Tauri
        tracing::info!("Application state created and managed.");

        // --- Spawn Background Tasks ---
        tauri::async_runtime::spawn(async move {
            backup::spawn_backup_check_task(pool.clone(), app_handle.clone());
        });
        tracing::info!("Background task(s) spawned.");

        Ok(())
    });

    // --- Register Commands and Run ---
    final_builder
        .invoke_handler(tauri::generate_handler![
            commands::admin_commands::login_admin,
            commands::membership_commands::add_membership_type,
            commands::membership_commands::get_all_membership_types,
            commands::membership_commands::delete_membership_type,
            commands::member_commands::add_member,
            commands::member_commands::get_members_with_memberships_paginated,
            commands::member_commands::get_member_by_id_with_membership,
            commands::member_commands::save_member_with_membership,
        ])
        // --- Optional: Add Plugins ---
        // .plugin(tauri_plugin_store::Builder::default().build()) // Example
        // --- Run the App ---
        // generate_context!() loads configuration from Tauri.toml/tauri.config.json
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application"); // Use expect for fatal errors on startup
}
