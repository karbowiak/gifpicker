use crate::commands::{CommandError, CommandResult};
use std::path::PathBuf;
use tokio::fs;

#[tauri::command]
pub async fn read_file_as_data_url(file_path: String) -> CommandResult<String> {
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(CommandError::NotFound(file_path));
    }

    let bytes = fs::read(&path).await?;

    let mime_type = match path.extension().and_then(|e| e.to_str()) {
        Some("gif") => "image/gif",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        _ => "application/octet-stream",
    };

    let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}
