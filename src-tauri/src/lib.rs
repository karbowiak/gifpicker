pub mod commands;
pub mod config;
pub mod db;
pub mod models;
pub mod services;

use commands::AppState;
use db::Database;
use services::Downloader;
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_drag::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // macOS: keep the app out of the dock — it's a menu-bar utility.
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            let db_path = app_dir.join("data").join("gifpicker.db");
            let db = tauri::async_runtime::block_on(async {
                let database = Database::new(db_path)
                    .await
                    .expect("Failed to initialize database");
                database
                    .run_migrations()
                    .await
                    .expect("Failed to run migrations");
                database
            });

            let media_dir = app_dir.join("media");
            let downloader = Downloader::new(media_dir).expect("Failed to initialize downloader");

            // Stable per-install UUID sent to Klipy for ad attribution. Generated
            // on first launch and persisted in the settings table.
            let customer_id = tauri::async_runtime::block_on(async {
                db.settings()
                    .get_or_create_customer_id()
                    .await
                    .expect("Failed to load or generate customer_id")
            });

            app.manage(AppState::new(
                Arc::new(db),
                Arc::new(downloader),
                customer_id,
            ));

            // Tray menu
            let show_item = MenuItemBuilder::with_id("show", "Show GIF Picker").build(app)?;
            let settings_item = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .item(&settings_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let tray_builder = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("GIF Picker")
                .menu(&menu)
                .show_menu_on_left_click(false);

            // macOS template icon adapts to light/dark menu bars.
            #[cfg(target_os = "macos")]
            let tray_builder = tray_builder.icon_as_template(true);

            let _tray = tray_builder
                .on_menu_event(|app, event| match event.id.as_ref() {
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
                            let _ = window.emit("open-settings", ());
                        }
                    }
                    "quit" => std::process::exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // Only respond to left-click release (Up) to avoid toggling on mouse-down.
                    // Right-click is handled by the menu via show_menu_on_left_click(false).
                    if let TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        commands::hotkey::toggle_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // Register the user's saved hotkey on startup, and apply the
            // persisted always-on-top preference to the window.
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Some(state) = app_handle.try_state::<AppState>() {
                    if let Ok(settings) = state.db.settings().get().await {
                        if let Ok(shortcut) = settings.hotkey.parse::<Shortcut>() {
                            let _ = app_handle.global_shortcut().on_shortcut(
                                shortcut,
                                |app, _shortcut, event| {
                                    if event.state == ShortcutState::Pressed {
                                        commands::hotkey::toggle_main_window(app);
                                    }
                                },
                            );
                        }

                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.set_always_on_top(settings.always_on_top);
                        }
                    }
                }
            });

            // Closing the window hides it instead of quitting — the app lives in the tray.
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();

                        #[cfg(target_os = "macos")]
                        {
                            use objc2::runtime::AnyObject;
                            use objc2::{class, msg_send};

                            // Window event handlers run on the main thread.
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
            // Favorites
            commands::get_all_favorites,
            commands::get_favorite_by_id,
            commands::add_favorite,
            commands::add_klipy_favorite,
            commands::update_favorite,
            commands::delete_favorite,
            commands::increment_use_count,
            commands::import_local_file,
            // Search
            commands::search_local,
            commands::search_klipy,
            commands::search_combined,
            commands::get_klipy_trending,
            commands::get_klipy_categories,
            commands::get_autocomplete,
            commands::get_search_suggestions,
            commands::download_gif_temp,
            // Settings
            commands::get_settings,
            commands::save_settings,
            commands::update_setting,
            // Clipboard
            commands::copy_image_to_clipboard,
            commands::copy_text_to_clipboard,
            commands::copy_file_path_to_clipboard,
            commands::get_clipboard_text,
            // Files
            commands::read_file_as_data_url,
            // System
            commands::open_url,
            // Window
            commands::close_window,
            commands::show_window,
            commands::toggle_window,
            commands::set_always_on_top,
            // Hotkey
            commands::register_hotkey,
            commands::unregister_hotkey,
            commands::unregister_all_hotkeys,
            commands::is_hotkey_registered,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            // Closing all windows shouldn't quit the app — it lives in the tray.
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
