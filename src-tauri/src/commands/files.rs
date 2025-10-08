use std::path::PathBuf;
use tokio::fs;

#[tauri::command]
pub async fn read_file_as_data_url(file_path: String) -> Result<String, String> {
    let path = PathBuf::from(&file_path);

    // Verify file exists
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Read file bytes
    let bytes = fs::read(&path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Detect MIME type based on extension
    let mime_type = match path.extension().and_then(|e| e.to_str()) {
        Some("gif") => "image/gif",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        _ => "application/octet-stream",
    };

    // Encode as base64
    let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);

    // Return as data URL
    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}
