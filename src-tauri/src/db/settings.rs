use anyhow::{Context, Result};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::Settings;

/// Settings-table key for the per-install Klipy `customer_id`. Not part of
/// `Settings` because it's never user-editable — generated once, kept forever.
const CUSTOMER_ID_KEY: &str = "klipy_customer_id";

pub struct SettingsDb<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SettingsDb<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Read the persisted Klipy customer_id, generating one on first call.
    /// Stable across launches; reset only by clearing the settings table.
    pub async fn get_or_create_customer_id(&self) -> Result<String> {
        let existing: Option<(String,)> =
            sqlx::query_as("SELECT value FROM settings WHERE key = ?")
                .bind(CUSTOMER_ID_KEY)
                .fetch_optional(self.pool)
                .await
                .context("Failed to read customer_id")?;

        if let Some((value,)) = existing {
            // Stored as a JSON string (matches the rest of the table's format).
            if let Ok(id) = serde_json::from_str::<String>(&value) {
                return Ok(id);
            }
        }

        let id = Uuid::new_v4().to_string();
        self.update_key(CUSTOMER_ID_KEY, serde_json::to_string(&id)?)
            .await?;
        Ok(id)
    }

    pub async fn get(&self) -> Result<Settings> {
        let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM settings")
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
                "hotkey" => {
                    settings.hotkey = serde_json::from_str(&value).unwrap_or(settings.hotkey)
                }
                "window_width" => {
                    settings.window_width =
                        serde_json::from_str(&value).unwrap_or(settings.window_width)
                }
                "window_height" => {
                    settings.window_height =
                        serde_json::from_str(&value).unwrap_or(settings.window_height)
                }
                "max_item_width" => {
                    settings.max_item_width =
                        serde_json::from_str(&value).unwrap_or(settings.max_item_width)
                }
                "close_after_selection" => {
                    settings.close_after_selection =
                        serde_json::from_str(&value).unwrap_or(settings.close_after_selection)
                }
                "launch_at_startup" => {
                    settings.launch_at_startup =
                        serde_json::from_str(&value).unwrap_or(settings.launch_at_startup)
                }
                "theme" => settings.theme = serde_json::from_str(&value).unwrap_or(settings.theme),
                "clipboard_mode" => {
                    settings.clipboard_mode =
                        serde_json::from_str(&value).unwrap_or(settings.clipboard_mode)
                }
                "clipboard_format" => {
                    settings.clipboard_format =
                        serde_json::from_str(&value).unwrap_or(settings.clipboard_format)
                }
                "show_ads" => {
                    settings.show_ads = serde_json::from_str(&value).unwrap_or(settings.show_ads)
                }
                "tile_size" => {
                    settings.tile_size =
                        serde_json::from_str(&value).unwrap_or(settings.tile_size)
                }
                "always_on_top" => {
                    settings.always_on_top =
                        serde_json::from_str(&value).unwrap_or(settings.always_on_top)
                }
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
            ("hotkey", serde_json::to_string(&settings.hotkey)?),
            (
                "window_width",
                serde_json::to_string(&settings.window_width)?,
            ),
            (
                "window_height",
                serde_json::to_string(&settings.window_height)?,
            ),
            (
                "max_item_width",
                serde_json::to_string(&settings.max_item_width)?,
            ),
            (
                "close_after_selection",
                serde_json::to_string(&settings.close_after_selection)?,
            ),
            (
                "launch_at_startup",
                serde_json::to_string(&settings.launch_at_startup)?,
            ),
            ("theme", serde_json::to_string(&settings.theme)?),
            (
                "clipboard_mode",
                serde_json::to_string(&settings.clipboard_mode)?,
            ),
            (
                "clipboard_format",
                serde_json::to_string(&settings.clipboard_format)?,
            ),
            ("show_ads", serde_json::to_string(&settings.show_ads)?),
            ("tile_size", serde_json::to_string(&settings.tile_size)?),
            (
                "always_on_top",
                serde_json::to_string(&settings.always_on_top)?,
            ),
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
        settings.window_width = 1024;
        settings.theme = Theme::Dark;

        settings_db.save(&settings).await.unwrap();

        let retrieved = settings_db.get().await.unwrap();
        assert_eq!(retrieved.window_width, 1024);
        assert_eq!(retrieved.theme, Theme::Dark);
    }

    #[tokio::test]
    async fn test_update_key() {
        let (db, _temp) = create_test_db().await;
        let settings_db = SettingsDb::new(db.pool());

        settings_db
            .update_key("hotkey", serde_json::to_string("Ctrl+Shift+G").unwrap())
            .await
            .unwrap();

        let settings = settings_db.get().await.unwrap();
        assert_eq!(settings.hotkey, "Ctrl+Shift+G");
    }
}
