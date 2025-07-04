// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use gym_manager_lib::{backup, commands, config, db, utils, AppState};
use tauri::Manager;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter("gym_manager=debug,tauri=info,sqlx=debug")
        .with_writer(std::io::stdout)
        .init();
    println!("=== Application starting ===");

    // Initialize the Tauri application builder
    let builder = tauri::Builder::default().plugin(tauri_plugin_process::init());

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
        let app_state = AppState::new(pool.clone(), settings);
        app.manage(app_state); // Register the state with Tauri
        tracing::info!("Application state created and managed.");

        // --- Spawn Background Tasks ---
        let handle_for_membership_check = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            utils::spawn_membership_check_task(handle_for_membership_check);
        });

        let handle_for_backup_check = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            backup::spawn_backup_check_task(handle_for_backup_check);
        });
        tracing::info!("Background task(s) spawned.");

        Ok(())
    });

    // --- Register Commands and Run ---
    final_builder
        .invoke_handler(tauri::generate_handler![
            commands::admin_commands::login,
            commands::admin_commands::get_app_settings,
            commands::admin_commands::update_app_settings,
            commands::admin_commands::get_all_users,
            commands::admin_commands::get_user_by_id,
            commands::admin_commands::change_user_password,
            commands::admin_commands::delete_user,
            commands::admin_commands::get_remote_backup_metadata,
            commands::admin_commands::trigger_backup,
            commands::admin_commands::save_user,
            backup::restore_from_backup,
            commands::membership_type_commands::add_membership_type,
            commands::membership_type_commands::get_all_membership_types,
            commands::membership_type_commands::get_membership_type_by_id,
            commands::membership_type_commands::update_membership_type,
            commands::membership_type_commands::delete_membership_type,
            commands::member_commands::add_member,
            commands::member_commands::get_member_by_id,
            commands::member_commands::delete_member,
            commands::member_commands::update_member,
            commands::member_commands::get_members_with_memberships_paginated,
            commands::member_commands::get_member_by_id_with_membership,
            commands::membership_commands::get_all_memberships_for_member,
            commands::membership_commands::save_membership,
            commands::membership_commands::delete_membership,
            commands::membership_commands::get_membership_by_id,
            commands::entry_log_commands::process_scan,
            commands::entry_log_commands::process_scan_single,
            commands::entry_log_commands::get_recent_entry_logs,
            commands::entry_log_commands::get_entry_logs,
            commands::entry_log_commands::delete_entry_log,
            commands::entry_log_commands::get_member_entry_logs,
            commands::entry_log_commands::delete_entry_logs,
            commands::analytics_commands::get_membership_type_distribution,
            commands::analytics_commands::get_revenue_by_membership_type,
            commands::analytics_commands::get_daily_hourly_visit_count,
            commands::analytics_commands::get_active_memberships_over_time,
        ])
        // --- Optional: Add Plugins ---
        .plugin(tauri_plugin_updater::Builder::default().build())
        // --- Run the App ---
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application"); // Use expect for fatal errors on startup
}
