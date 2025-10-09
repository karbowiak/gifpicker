use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::models::Settings;

pub struct SettingsDb<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SettingsDb<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get(&self) -> Result<Settings> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT key, value FROM settings"
        )
        .fetch_all(self.pool)
        .await
        .context("Failed to fetch settings")?;

        if rows.is_empty() {
            // Return default settings if none exist
            return Ok(Settings::default());
        }

        // Deserialize from key-value pairs
        let mut settings = Settings::default();
        for (key, value) in rows {
            match key.as_str() {
                "giphy_api_key" => settings.giphy_api_key = serde_json::from_str(&value).ok(),
                "hotkey" => settings.hotkey = serde_json::from_str(&value).unwrap_or(settings.hotkey),
                "window_width" => settings.window_width = serde_json::from_str(&value).unwrap_or(settings.window_width),
                "window_height" => settings.window_height = serde_json::from_str(&value).unwrap_or(settings.window_height),
                "max_item_width" => settings.max_item_width = serde_json::from_str(&value).unwrap_or(settings.max_item_width),
                "close_after_selection" => settings.close_after_selection = serde_json::from_str(&value).unwrap_or(settings.close_after_selection),
                "launch_at_startup" => settings.launch_at_startup = serde_json::from_str(&value).unwrap_or(settings.launch_at_startup),
                "theme" => settings.theme = serde_json::from_str(&value).unwrap_or(settings.theme),
                "clipboard_mode" => settings.clipboard_mode = serde_json::from_str(&value).unwrap_or(settings.clipboard_mode),
                _ => {}
            }
        }

        Ok(settings)
    }

    pub async fn save(&self, settings: &Settings) -> Result<()> {
        // Delete all existing settings
        sqlx::query("DELETE FROM settings")
            .execute(self.pool)
            .await
            .context("Failed to clear settings")?;

        // Insert all settings
        let pairs = vec![
            ("giphy_api_key", serde_json::to_string(&settings.giphy_api_key)?),
            ("hotkey", serde_json::to_string(&settings.hotkey)?),
            ("window_width", serde_json::to_string(&settings.window_width)?),
            ("window_height", serde_json::to_string(&settings.window_height)?),
            ("max_item_width", serde_json::to_string(&settings.max_item_width)?),
            ("close_after_selection", serde_json::to_string(&settings.close_after_selection)?),
            ("launch_at_startup", serde_json::to_string(&settings.launch_at_startup)?),
            ("theme", serde_json::to_string(&settings.theme)?),
            ("clipboard_mode", serde_json::to_string(&settings.clipboard_mode)?),
        ];

        for (key, value) in pairs {
            sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?)")
                .bind(key)
                .bind(value)
                .execute(self.pool)
                .await
                .context("Failed to insert setting")?;
        }

        Ok(())
    }

    pub async fn update_key(&self, key: &str, value: String) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO settings (key, value) VALUES (?, ?)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(self.pool)
        .await
        .context("Failed to update setting")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::models::Theme;
    use tempfile::TempDir;

    async fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path).await.unwrap();
        db.run_migrations().await.unwrap();
        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_get_default_settings() {
        let (db, _temp) = create_test_db().await;
        let settings_db = SettingsDb::new(db.pool());

        let settings = settings_db.get().await.unwrap();
        assert_eq!(settings.window_width, 800);
        assert_eq!(settings.window_height, 600);
    }

    #[tokio::test]
    async fn test_save_and_get_settings() {
        let (db, _temp) = create_test_db().await;
        let settings_db = SettingsDb::new(db.pool());

        let mut settings = Settings::default();
        settings.giphy_api_key = Some("test_key_123".to_string());
        settings.window_width = 1024;
        settings.theme = Theme::Dark;

        settings_db.save(&settings).await.unwrap();

        let retrieved = settings_db.get().await.unwrap();
        assert_eq!(retrieved.giphy_api_key, Some("test_key_123".to_string()));
        assert_eq!(retrieved.window_width, 1024);
        assert_eq!(retrieved.theme, Theme::Dark);
    }

    #[tokio::test]
    async fn test_update_key() {
        let (db, _temp) = create_test_db().await;
        let settings_db = SettingsDb::new(db.pool());

        settings_db.update_key("hotkey", serde_json::to_string("Ctrl+Shift+G").unwrap()).await.unwrap();

        let settings = settings_db.get().await.unwrap();
        assert_eq!(settings.hotkey, "Ctrl+Shift+G");
    }
}
