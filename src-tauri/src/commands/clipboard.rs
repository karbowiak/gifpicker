use crate::services::ClipboardManager;
use std::path::PathBuf;

#[tauri::command]
pub async fn copy_image_to_clipboard(
    file_path: String,
) -> Result<(), String> {
    let path = PathBuf::from(file_path);

    let mut clipboard = ClipboardManager::new()
        .map_err(|e| format!("Failed to initialize clipboard: {}", e))?;

    clipboard.copy_image(&path)
        .map_err(|e| format!("Failed to copy image to clipboard: {}", e))
}

#[tauri::command]
pub async fn copy_text_to_clipboard(
    text: String,
) -> Result<(), String> {
    let mut clipboard = ClipboardManager::new()
        .map_err(|e| format!("Failed to initialize clipboard: {}", e))?;

    clipboard.copy_text(&text)
        .map_err(|e| format!("Failed to copy text to clipboard: {}", e))
}

#[tauri::command]
pub async fn copy_file_path_to_clipboard(
    file_path: String,
) -> Result<(), String> {
    let path = PathBuf::from(file_path);

    let mut clipboard = ClipboardManager::new()
        .map_err(|e| format!("Failed to initialize clipboard: {}", e))?;

    clipboard.copy_file_path(&path)
        .map_err(|e| format!("Failed to copy file path to clipboard: {}", e))
}

#[tauri::command]
pub async fn get_clipboard_text() -> Result<String, String> {
    let mut clipboard = ClipboardManager::new()
        .map_err(|e| format!("Failed to initialize clipboard: {}", e))?;

    clipboard.get_text()
        .map_err(|e| format!("Failed to get clipboard text: {}", e))
}
