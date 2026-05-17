use crate::commands::CommandResult;
use tauri::{AppHandle, Emitter, Manager};

#[tauri::command]
pub async fn close_window(app: AppHandle) -> CommandResult<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    window.hide()?;

    // On macOS, hide+deactivate the whole app so focus returns to whoever the
    // user was talking to (Discord, Slack, etc.) before they popped the picker.
    #[cfg(target_os = "macos")]
    {
        let (tx, rx) = tokio::sync::oneshot::channel();

        app.run_on_main_thread(move || {
            use objc2::runtime::AnyObject;
            use objc2::{class, msg_send};

            unsafe {
                let app = class!(NSApplication);
                let shared: *mut AnyObject = msg_send![app, sharedApplication];
                let _: () = msg_send![shared, hide: shared];
                let _: () = msg_send![shared, deactivate];
            }
            let _ = tx.send(());
        })?;

        let _ = rx.await;
    }

    Ok(())
}

#[tauri::command]
pub async fn show_window(app: AppHandle) -> CommandResult<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    window.show()?;
    window.set_focus()?;
    let _ = window.emit("clear-search", ());
    let _ = window.emit("focus-search", ());
    Ok(())
}

#[tauri::command]
pub async fn toggle_window(app: AppHandle) -> CommandResult<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    if window.is_visible()? {
        window.hide()?;
    } else {
        window.show()?;
        window.set_focus()?;
        let _ = window.emit("focus-search", ());
    }
    Ok(())
}

#[tauri::command]
pub async fn set_always_on_top(app: AppHandle, value: bool) -> CommandResult<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    window.set_always_on_top(value)?;
    Ok(())
}
