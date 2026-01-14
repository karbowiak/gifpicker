use tauri::command;

#[command]
pub async fn open_url(url: String) -> Result<(), String> {
    open::that(url).map_err(|e| e.to_string())
}
