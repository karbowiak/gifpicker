use crate::commands::{AppState, CommandResult};
use crate::models::Settings;

#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, AppState>) -> CommandResult<Settings> {
    Ok(state.db.settings().get().await?)
}

#[tauri::command]
pub async fn save_settings(
    settings: Settings,
    state: tauri::State<'_, AppState>,
) -> CommandResult<()> {
    state.db.settings().save(&settings).await?;
    Ok(())
}

#[tauri::command]
pub async fn update_setting(
    key: String,
    value: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult<()> {
    state.db.settings().update_key(&key, value).await?;
    Ok(())
}
