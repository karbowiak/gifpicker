use crate::commands::{CommandError, CommandResult};
use crate::services::ClipboardManager;
use std::path::PathBuf;

fn clipboard() -> CommandResult<ClipboardManager> {
    ClipboardManager::new().map_err(|e| CommandError::Clipboard(e.to_string()))
}

#[tauri::command]
pub fn copy_image_to_clipboard(file_path: String) -> CommandResult<()> {
    let path = PathBuf::from(file_path);
    clipboard()?
        .copy_image(&path)
        .map_err(|e| CommandError::Clipboard(e.to_string()))
}

#[tauri::command]
pub fn copy_text_to_clipboard(text: String) -> CommandResult<()> {
    clipboard()?
        .copy_text(&text)
        .map_err(|e| CommandError::Clipboard(e.to_string()))
}

#[tauri::command]
pub fn copy_file_path_to_clipboard(file_path: String) -> CommandResult<()> {
    let path = PathBuf::from(file_path);
    clipboard()?
        .copy_file_path(&path)
        .map_err(|e| CommandError::Clipboard(e.to_string()))
}

#[tauri::command]
pub fn get_clipboard_text() -> CommandResult<String> {
    clipboard()?
        .get_text()
        .map_err(|e| CommandError::Clipboard(e.to_string()))
}
