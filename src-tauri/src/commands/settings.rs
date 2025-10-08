use crate::commands::AppState;
use crate::db::SettingsDb;
use crate::models::Settings;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn get_settings(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Settings, String> {
    let state = state.lock().await;
    let settings_db = SettingsDb::new(state.db.pool());

    settings_db.get()
        .await
        .map_err(|e| format!("Failed to get settings: {}", e))
}

#[tauri::command]
pub async fn save_settings(
    settings: Settings,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let state = state.lock().await;
    let settings_db = SettingsDb::new(state.db.pool());

    settings_db.save(&settings)
        .await
        .map_err(|e| format!("Failed to save settings: {}", e))
}

#[tauri::command]
pub async fn update_setting(
    key: String,
    value: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let state = state.lock().await;
    let settings_db = SettingsDb::new(state.db.pool());

    settings_db.update_key(&key, value)
        .await
        .map_err(|e| format!("Failed to update setting: {}", e))
}
