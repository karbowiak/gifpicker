use crate::commands::{CommandError, CommandResult};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

fn parse_shortcut(hotkey: &str) -> CommandResult<Shortcut> {
    hotkey
        .parse::<Shortcut>()
        .map_err(|e| CommandError::Hotkey(format!("invalid hotkey '{}': {}", hotkey, e)))
}

#[tauri::command]
pub async fn register_hotkey(app: AppHandle, hotkey: String) -> CommandResult<()> {
    // Replace any previously-registered shortcut so duplicate registrations don't error.
    let _ = unregister_all_hotkeys(app.clone()).await;

    let shortcut = parse_shortcut(&hotkey)?;

    app.global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            toggle_main_window(app);
        })
        .map_err(|e| CommandError::Hotkey(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn unregister_hotkey(app: AppHandle, hotkey: String) -> CommandResult<()> {
    let shortcut = parse_shortcut(&hotkey)?;
    app.global_shortcut()
        .unregister(shortcut)
        .map_err(|e| CommandError::Hotkey(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn unregister_all_hotkeys(app: AppHandle) -> CommandResult<()> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| CommandError::Hotkey(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn is_hotkey_registered(app: AppHandle, hotkey: String) -> CommandResult<bool> {
    let shortcut = parse_shortcut(&hotkey)?;
    Ok(app.global_shortcut().is_registered(shortcut))
}

/// Toggle the main window: hide-if-visible / show-if-hidden, mirroring the
/// tray-icon click behavior. Used by the global hotkey handler.
pub(crate) fn toggle_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        deactivate_app();
    } else {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.center();
        let _ = window.emit("clear-search", ());
        let _ = window.emit("focus-search", ());
    }
}

#[cfg(target_os = "macos")]
fn deactivate_app() {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};

    // Safe: hotkey/tray callbacks run on the main thread.
    unsafe {
        let app = class!(NSApplication);
        let shared: *mut AnyObject = msg_send![app, sharedApplication];
        let _: () = msg_send![shared, deactivate];
    }
}

#[cfg(not(target_os = "macos"))]
fn deactivate_app() {}
