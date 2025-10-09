pub mod models;
pub mod db;
pub mod services;
pub mod commands;

use commands::AppState;
use db::Database;
use services::Downloader;
use std::sync::Arc;
use tauri::{Manager, Emitter};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
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
            // Set activation policy on macOS to hide from dock
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

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

            // Build tray menu
            let show_item = MenuItemBuilder::with_id("show", "Show GIF Picker").build(app)?;
            let settings_item = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .item(&settings_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Build tray icon - only create once per app lifecycle
            let tray_builder = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("GIF Picker")
                .menu(&menu);

            // On macOS, set the icon as a template so it adapts to light/dark mode
            #[cfg(target_os = "macos")]
            let tray_builder = tray_builder.icon_as_template(true);

            let _tray = tray_builder
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("clear-search", ());
                                let _ = window.emit("focus-search", ());
                            }
                        }
                        "settings" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                // Emit event to open settings (handled by frontend)
                                let _ = window.emit("open-settings", ());
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                // Hide and deactivate
                                let _ = window.hide();
                                #[cfg(target_os = "macos")]
                                {
                                    use objc2::runtime::AnyObject;
                                    use objc2::{class, msg_send};

                                    // Tray event handlers run on main thread, safe to call directly
                                    unsafe {
                                        let app = class!(NSApplication);
                                        let shared: *mut AnyObject = msg_send![app, sharedApplication];
                                        let _: () = msg_send![shared, deactivate];
                                    }
                                }
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("clear-search", ());
                                let _ = window.emit("focus-search", ());
                            }
                        }
                    }
                })
                .build(app)?;

            // Register initial hotkey from settings
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Get state
                if let Some(state) = app_handle.try_state::<Arc<Mutex<AppState>>>() {
                    let state = state.lock().await;

                    // Get settings from database
                    if let Ok(settings) = state.db.settings().get().await {
                        let hotkey = settings.hotkey;

                        // Register the hotkey
                        if let Ok(shortcut) = hotkey.parse::<Shortcut>() {
                            let _ = app_handle.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
                                // Only respond to key press, not release
                                if event.state == ShortcutState::Pressed {
                                    if let Some(window) = app.get_webview_window("main") {
                                        if window.is_visible().unwrap_or(false) {
                                            // Hide and deactivate
                                            let _ = window.hide();
                                            #[cfg(target_os = "macos")]
                                            {
                                                use objc2::runtime::AnyObject;
                                                use objc2::{class, msg_send};

                                                // Hotkey handlers run on main thread, safe to call directly
                                                unsafe {
                                                    let app = class!(NSApplication);
                                                    let shared: *mut AnyObject = msg_send![app, sharedApplication];
                                                    let _: () = msg_send![shared, deactivate];
                                                }
                                            }
                                        } else {
                                            let _ = window.show();
                                            let _ = window.set_focus();
                                            let _ = window.center();
                                            let _ = window.emit("clear-search", ());
                                            let _ = window.emit("focus-search", ());
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
            });

            // Handle window close event - hide instead of quit
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // Prevent the window from closing
                        api.prevent_close();
                        // Hide the window instead
                        let _ = window_clone.hide();
                        // Deactivate on macOS to return focus
                        // Window event handlers run on main thread, safe to call directly
                        #[cfg(target_os = "macos")]
                        {
                            use objc2::runtime::AnyObject;
                            use objc2::{class, msg_send};

                            unsafe {
                                let app = class!(NSApplication);
                                let shared: *mut AnyObject = msg_send![app, sharedApplication];
                                let _: () = msg_send![shared, deactivate];
                            }
                        }
                    }
                });
            }

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
            commands::download_gif_temp,
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
            // Window commands
            commands::close_window,
            commands::show_window,
            commands::toggle_window,
            // Hotkey commands
            commands::register_hotkey,
            commands::unregister_hotkey,
            commands::unregister_all_hotkeys,
            commands::is_hotkey_registered,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            // Prevent the app from exiting when all windows are closed
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
