use tauri::{AppHandle, Emitter, Manager};

#[tauri::command]
pub async fn close_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        // Hide the window FIRST
        window.hide().map_err(|e| e.to_string())?;

        // On macOS, hide and deactivate the app so focus returns to previous app
        #[cfg(target_os = "macos")]
        {
            // Use a channel to wait for the main thread operation to complete
            let (tx, rx) = tokio::sync::oneshot::channel();

            app.run_on_main_thread(move || {
                use objc2::runtime::AnyObject;
                use objc2::{class, msg_send};

                unsafe {
                    let app = class!(NSApplication);
                    let shared: *mut AnyObject = msg_send![app, sharedApplication];
                    // Hide the entire app and return focus
                    let _: () = msg_send![shared, hide: shared];
                    // Then deactivate for good measure
                    let _: () = msg_send![shared, deactivate];
                }

                // Signal completion
                let _ = tx.send(());
            }).map_err(|e| e.to_string())?;

            // Wait for hide/deactivate to complete
            let _ = rx.await;
        }

        // Don't clear search here - let the frontend handle it when showing again
    }
    Ok(())
}

#[tauri::command]
pub async fn show_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        // Clear search and reset selection when showing window
        let _ = window.emit("clear-search", ());
        let _ = window.emit("focus-search", ());
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().map_err(|e| e.to_string())? {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
            // Emit event to focus search field
            let _ = window.emit("focus-search", ());
        }
    }
    Ok(())
}
