pub mod models;
pub mod db;
pub mod services;
pub mod commands;

use commands::AppState;
use db::Database;
use services::Downloader;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!!!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Get app data directory
            let app_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");

            // Initialize database
            let db_path = app_dir.join("data").join("gifpicker.db");
            let db = tauri::async_runtime::block_on(async {
                let database = Database::new(db_path).await
                    .expect("Failed to initialize database");
                database.run_migrations().await
                    .expect("Failed to run migrations");
                database
            });

            // Initialize downloader
            let media_dir = app_dir.join("media");
            let downloader = Downloader::new(media_dir)
                .expect("Failed to initialize downloader");

            // Create app state
            let state = Arc::new(Mutex::new(AppState {
                db: Arc::new(db),
                downloader: Arc::new(downloader),
            }));

            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            // Favorites commands
            commands::get_all_favorites,
            commands::get_favorite_by_id,
            commands::add_favorite,
            commands::add_giphy_favorite,
            commands::update_favorite,
            commands::delete_favorite,
            commands::increment_use_count,
            commands::import_local_file,
            // Search commands
            commands::search_local,
            commands::search_giphy,
            commands::search_combined,
            commands::get_giphy_trending,
            commands::download_giphy_gif,
            // Settings commands
            commands::get_settings,
            commands::save_settings,
            commands::update_setting,
            // Clipboard commands
            commands::copy_image_to_clipboard,
            commands::copy_text_to_clipboard,
            commands::copy_file_path_to_clipboard,
            commands::get_clipboard_text,
            // File serving commands
            commands::read_file_as_data_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
