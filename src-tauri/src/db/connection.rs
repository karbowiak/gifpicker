use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;
use super::settings::SettingsDb;
use super::favorites::FavoritesDb;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }

        let db_url = format!("sqlite:{}", db_path.display());
        let options = SqliteConnectOptions::from_str(&db_url)?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("Failed to connect to database")?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        // Create migrations tracking table if it doesn't exist
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL
            )"
        )
        .execute(&self.pool)
        .await
        .context("Failed to create migrations table")?;

        // Define migrations
        let migrations = vec![
            (1, "001_initial", include_str!("../../migrations/001_initial.sql")),
            (2, "002_add_gif_url", include_str!("../../migrations/002_add_gif_url.sql")),
            (3, "003_add_clipboard_mode", include_str!("../../migrations/003_add_clipboard_mode.sql")),
        ];

        // Run each migration if not already applied
        for (version, name, sql) in migrations {
            // Check if migration already applied
            let result: Option<(i64,)> = sqlx::query_as(
                "SELECT version FROM _migrations WHERE version = ?"
            )
            .bind(version)
            .fetch_optional(&self.pool)
            .await
            .context(format!("Failed to check migration {}", name))?;

            if result.is_none() {
                // Run migration
                sqlx::query(sql)
                    .execute(&self.pool)
                    .await
                    .context(format!("Failed to run migration {}", name))?;

                // Record migration as applied
                sqlx::query(
                    "INSERT INTO _migrations (version, name, applied_at) VALUES (?, ?, datetime('now'))"
                )
                .bind(version)
                .bind(name)
                .execute(&self.pool)
                .await
                .context(format!("Failed to record migration {}", name))?;

                println!("Applied migration: {}", name);
            }
        }

        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn settings(&self) -> SettingsDb<'_> {
        SettingsDb::new(&self.pool)
    }

    pub fn favorites(&self) -> FavoritesDb<'_> {
        FavoritesDb::new(&self.pool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path).await.unwrap();
        db.run_migrations().await.unwrap();
        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_database_creation() {
        let (db, _temp) = create_test_db().await;

        // Test that we can execute a simple query
        let result: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(db.pool())
            .await
            .unwrap();

        assert_eq!(result.0, 1);
    }

    #[tokio::test]
    async fn test_migrations_create_tables() {
        let (db, _temp) = create_test_db().await;

        // Check that favorites table exists
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='favorites'"
        )
        .fetch_one(db.pool())
        .await
        .unwrap();

        assert_eq!(result.0, 1);

        // Check that settings table exists
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='settings'"
        )
        .fetch_one(db.pool())
        .await
        .unwrap();

        assert_eq!(result.0, 1);
    }
}
