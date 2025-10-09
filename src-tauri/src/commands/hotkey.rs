use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[tauri::command]
pub async fn register_hotkey(app: AppHandle, hotkey: String) -> Result<(), String> {
    // Unregister any existing hotkeys first
    let _ = unregister_all_hotkeys(app.clone()).await;

    // Parse the hotkey string
    let shortcut = hotkey.parse::<Shortcut>()
        .map_err(|e| format!("Invalid hotkey format: {}", e))?;

    // Register the new hotkey
    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _shortcut, event| {
            // Only respond to key press, not release
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                // Toggle window visibility when hotkey is pressed
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        // Hide and deactivate
                        let _ = window.hide();
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
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.center();
                        let _ = window.emit("clear-search", ());
                        let _ = window.emit("focus-search", ());
                    }
                }
            }
        })
        .map_err(|e| format!("Failed to register hotkey: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn unregister_hotkey(app: AppHandle, hotkey: String) -> Result<(), String> {
    let shortcut = hotkey.parse::<Shortcut>()
        .map_err(|e| format!("Invalid hotkey format: {}", e))?;

    app.global_shortcut()
        .unregister(shortcut)
        .map_err(|e| format!("Failed to unregister hotkey: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn unregister_all_hotkeys(app: AppHandle) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| format!("Failed to unregister all hotkeys: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn is_hotkey_registered(app: AppHandle, hotkey: String) -> Result<bool, String> {
    let shortcut = hotkey.parse::<Shortcut>()
        .map_err(|e| format!("Invalid hotkey format: {}", e))?;

    Ok(app.global_shortcut().is_registered(shortcut))
}
