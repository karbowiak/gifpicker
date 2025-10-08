use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::models::{Favorite, MediaType, Source};

pub struct FavoritesDb<'a> {
    pool: &'a SqlitePool,
}

impl<'a> FavoritesDb<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, favorite: &Favorite) -> Result<i64> {
        let tags_json = serde_json::to_string(&favorite.tags)?;
        let custom_tags_json = serde_json::to_string(&favorite.custom_tags)?;
        let source = favorite.source.as_ref().map(|s| s.to_string());
        let created_at = favorite.created_at.to_rfc3339();
        let last_used = favorite.last_used.map(|dt| dt.to_rfc3339());

        let result = sqlx::query(
            r#"
            INSERT INTO favorites (
                filename, filepath, gif_url, media_type, source, source_id, source_url,
                tags, custom_tags, description, width, height, file_size,
                created_at, last_used, use_count
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&favorite.filename)
        .bind(&favorite.filepath)
        .bind(&favorite.gif_url)
        .bind(favorite.media_type.to_string())
        .bind(source)
        .bind(&favorite.source_id)
        .bind(&favorite.source_url)
        .bind(tags_json)
        .bind(custom_tags_json)
        .bind(&favorite.description)
        .bind(favorite.width)
        .bind(favorite.height)
        .bind(favorite.file_size)
        .bind(created_at)
        .bind(last_used)
        .bind(favorite.use_count)
        .execute(self.pool)
        .await
        .context("Failed to insert favorite")?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_by_id(&self, id: i64) -> Result<Option<Favorite>> {
        let row = sqlx::query_as::<_, FavoriteRow>(
            r#"
            SELECT id, filename, filepath, gif_url, media_type, source, source_id, source_url,
                   tags, custom_tags, description, width, height, file_size,
                   created_at, last_used, use_count
            FROM favorites
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .context("Failed to fetch favorite")?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn get_all(&self) -> Result<Vec<Favorite>> {
        let rows = sqlx::query_as::<_, FavoriteRow>(
            r#"
            SELECT id, filename, filepath, gif_url, media_type, source, source_id, source_url,
                   tags, custom_tags, description, width, height, file_size,
                   created_at, last_used, use_count
            FROM favorites
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(self.pool)
        .await
        .context("Failed to fetch all favorites")?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Favorite>> {
        let search_term = format!("%{}%", query.to_lowercase());

        let rows = sqlx::query_as::<_, FavoriteRow>(
            r#"
            SELECT id, filename, filepath, gif_url, media_type, source, source_id, source_url,
                   tags, custom_tags, description, width, height, file_size,
                   created_at, last_used, use_count
            FROM favorites
            WHERE LOWER(filename) LIKE ?
               OR LOWER(tags) LIKE ?
               OR LOWER(custom_tags) LIKE ?
               OR LOWER(description) LIKE ?
            ORDER BY use_count DESC, created_at DESC
            "#,
        )
        .bind(&search_term)
        .bind(&search_term)
        .bind(&search_term)
        .bind(&search_term)
        .fetch_all(self.pool)
        .await
        .context("Failed to search favorites")?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update(&self, favorite: &Favorite) -> Result<()> {
        let id = favorite.id.context("Favorite must have an ID to update")?;
        let tags_json = serde_json::to_string(&favorite.tags)?;
        let custom_tags_json = serde_json::to_string(&favorite.custom_tags)?;
        let source = favorite.source.as_ref().map(|s| s.to_string());
        let last_used = favorite.last_used.map(|dt| dt.to_rfc3339());

        sqlx::query(
            r#"
            UPDATE favorites
            SET filename = ?, filepath = ?, gif_url = ?, media_type = ?, source = ?, source_id = ?,
                source_url = ?, tags = ?, custom_tags = ?, description = ?,
                width = ?, height = ?, file_size = ?, last_used = ?, use_count = ?
            WHERE id = ?
            "#,
        )
        .bind(&favorite.filename)
        .bind(&favorite.filepath)
        .bind(&favorite.gif_url)
        .bind(favorite.media_type.to_string())
        .bind(source)
        .bind(&favorite.source_id)
        .bind(&favorite.source_url)
        .bind(tags_json)
        .bind(custom_tags_json)
        .bind(&favorite.description)
        .bind(favorite.width)
        .bind(favorite.height)
        .bind(favorite.file_size)
        .bind(last_used)
        .bind(favorite.use_count)
        .bind(id)
        .execute(self.pool)
        .await
        .context("Failed to update favorite")?;

        Ok(())
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM favorites WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await
            .context("Failed to delete favorite")?;

        Ok(())
    }

    pub async fn increment_use_count(&self, id: i64) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE favorites
            SET use_count = use_count + 1, last_used = ?
            WHERE id = ?
            "#,
        )
        .bind(now)
        .bind(id)
        .execute(self.pool)
        .await
        .context("Failed to increment use count")?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct FavoriteRow {
    id: i64,
    filename: String,
    filepath: Option<String>,
    gif_url: Option<String>,
    media_type: String,
    source: Option<String>,
    source_id: Option<String>,
    source_url: Option<String>,
    tags: String,
    custom_tags: String,
    description: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
    file_size: Option<i64>,
    created_at: String,
    last_used: Option<String>,
    use_count: i32,
}

impl From<FavoriteRow> for Favorite {
    fn from(row: FavoriteRow) -> Self {
        let tags: Vec<String> = serde_json::from_str(&row.tags).unwrap_or_default();
        let custom_tags: Vec<String> = serde_json::from_str(&row.custom_tags).unwrap_or_default();
        let media_type: MediaType = row.media_type.parse().unwrap_or(MediaType::Gif);
        let source: Option<Source> = row.source.and_then(|s| s.parse().ok());
        let created_at = DateTime::parse_from_rfc3339(&row.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let last_used = row.last_used.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        });

        Favorite {
            id: Some(row.id),
            filename: row.filename,
            filepath: row.filepath,
            gif_url: row.gif_url,
            media_type,
            source,
            source_id: row.source_id,
            source_url: row.source_url,
            tags,
            custom_tags,
            description: row.description,
            width: row.width,
            height: row.height,
            file_size: row.file_size,
            created_at,
            last_used,
            use_count: row.use_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use tempfile::TempDir;

    async fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path).await.unwrap();
        db.run_migrations().await.unwrap();
        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_create_and_get_favorite() {
        let (db, _temp) = create_test_db().await;
        let favorites_db = FavoritesDb::new(db.pool());

        let favorite = Favorite::new(
            "test.gif".to_string(),
            Some("/path/to/test.gif".to_string()),
            MediaType::Gif,
        );

        let id = favorites_db.create(&favorite).await.unwrap();
        assert!(id > 0);

        let retrieved = favorites_db.get_by_id(id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.filename, "test.gif");
        assert_eq!(retrieved.filepath, Some("/path/to/test.gif".to_string()));
        assert_eq!(retrieved.media_type, MediaType::Gif);
    }

    #[tokio::test]
    async fn test_get_all_favorites() {
        let (db, _temp) = create_test_db().await;
        let favorites_db = FavoritesDb::new(db.pool());

        // Create multiple favorites
        for i in 0..3 {
            let favorite = Favorite::new(
                format!("test{}.gif", i),
                Some(format!("/path/to/test{}.gif", i)),
                MediaType::Gif,
            );
            favorites_db.create(&favorite).await.unwrap();
        }

        let all = favorites_db.get_all().await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_search_favorites() {
        let (db, _temp) = create_test_db().await;
        let favorites_db = FavoritesDb::new(db.pool());

        let favorite1 = Favorite::new(
            "funny_cat.gif".to_string(),
            Some("/path/to/funny_cat.gif".to_string()),
            MediaType::Gif,
        )
        .with_tags(vec!["cat".to_string(), "funny".to_string()]);

        let favorite2 = Favorite::new(
            "dog.gif".to_string(),
            Some("/path/to/dog.gif".to_string()),
            MediaType::Gif,
        )
        .with_tags(vec!["dog".to_string()]);

        favorites_db.create(&favorite1).await.unwrap();
        favorites_db.create(&favorite2).await.unwrap();

        // Search for cat
        let results = favorites_db.search("cat").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "funny_cat.gif");

        // Search for dog
        let results = favorites_db.search("dog").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "dog.gif");
    }

    #[tokio::test]
    async fn test_update_favorite() {
        let (db, _temp) = create_test_db().await;
        let favorites_db = FavoritesDb::new(db.pool());

        let favorite = Favorite::new(
            "test.gif".to_string(),
            Some("/path/to/test.gif".to_string()),
            MediaType::Gif,
        );

        let id = favorites_db.create(&favorite).await.unwrap();

        let mut retrieved = favorites_db.get_by_id(id).await.unwrap().unwrap();
        retrieved.custom_tags = vec!["awesome".to_string()];
        retrieved.description = Some("A test GIF".to_string());

        favorites_db.update(&retrieved).await.unwrap();

        let updated = favorites_db.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(updated.custom_tags, vec!["awesome".to_string()]);
        assert_eq!(updated.description, Some("A test GIF".to_string()));
    }

    #[tokio::test]
    async fn test_delete_favorite() {
        let (db, _temp) = create_test_db().await;
        let favorites_db = FavoritesDb::new(db.pool());

        let favorite = Favorite::new(
            "test.gif".to_string(),
            Some("/path/to/test.gif".to_string()),
            MediaType::Gif,
        );

        let id = favorites_db.create(&favorite).await.unwrap();
        favorites_db.delete(id).await.unwrap();

        let retrieved = favorites_db.get_by_id(id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_increment_use_count() {
        let (db, _temp) = create_test_db().await;
        let favorites_db = FavoritesDb::new(db.pool());

        let favorite = Favorite::new(
            "test.gif".to_string(),
            Some("/path/to/test.gif".to_string()),
            MediaType::Gif,
        );

        let id = favorites_db.create(&favorite).await.unwrap();

        favorites_db.increment_use_count(id).await.unwrap();
        favorites_db.increment_use_count(id).await.unwrap();

        let updated = favorites_db.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(updated.use_count, 2);
        assert!(updated.last_used.is_some());
    }
}
