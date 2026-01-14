use crate::db::{Database, FavoritesDb};
use crate::models::{Favorite, MediaType, Source};
use crate::services::Downloader;
use anyhow::Result;
use image::GenericImageView;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub db: Arc<Database>,
    pub downloader: Arc<Downloader>,
}

#[tauri::command]
pub async fn get_all_favorites(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<Favorite>, String> {
    let state = state.lock().await;
    let favorites_db = FavoritesDb::new(state.db.pool());

    favorites_db
        .get_all()
        .await
        .map_err(|e| format!("Failed to get favorites: {}", e))
}

#[tauri::command]
pub async fn get_favorite_by_id(
    id: i64,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Option<Favorite>, String> {
    let state = state.lock().await;
    let favorites_db = FavoritesDb::new(state.db.pool());

    favorites_db
        .get_by_id(id)
        .await
        .map_err(|e| format!("Failed to get favorite: {}", e))
}

#[tauri::command]
pub async fn add_favorite(
    favorite: Favorite,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<i64, String> {
    let state = state.lock().await;
    let favorites_db = FavoritesDb::new(state.db.pool());

    favorites_db
        .create(&favorite)
        .await
        .map_err(|e| format!("Failed to add favorite: {}", e))
}

#[tauri::command]
pub async fn update_favorite(
    favorite: Favorite,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let state = state.lock().await;
    let favorites_db = FavoritesDb::new(state.db.pool());

    favorites_db
        .update(&favorite)
        .await
        .map_err(|e| format!("Failed to update favorite: {}", e))
}

#[tauri::command]
pub async fn delete_favorite(
    id: i64,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let state = state.lock().await;
    let favorites_db = FavoritesDb::new(state.db.pool());

    // Get the favorite to delete its file (if it has one)
    if let Ok(Some(favorite)) = favorites_db.get_by_id(id).await {
        // Delete the GIF file if it exists locally
        if let Some(filepath) = &favorite.filepath {
            let path = std::path::PathBuf::from(filepath);
            if path.exists() {
                Downloader::delete_file(&path)
                    .await
                    .map_err(|e| format!("Failed to delete GIF file: {}", e))?;
            }
        }
        // Delete the MP4 file if it exists locally
        if let Some(mp4_filepath) = &favorite.mp4_filepath {
            let path = std::path::PathBuf::from(mp4_filepath);
            if path.exists() {
                Downloader::delete_file(&path)
                    .await
                    .map_err(|e| format!("Failed to delete MP4 file: {}", e))?;
            }
        }
    }

    favorites_db
        .delete(id)
        .await
        .map_err(|e| format!("Failed to delete favorite: {}", e))
}

#[tauri::command]
pub async fn increment_use_count(
    id: i64,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let state = state.lock().await;
    let favorites_db = FavoritesDb::new(state.db.pool());

    favorites_db
        .increment_use_count(id)
        .await
        .map_err(|e| format!("Failed to increment use count: {}", e))
}

#[tauri::command]
pub async fn import_local_file(
    file_path: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Favorite, String> {
    let state = state.lock().await;
    let source_path = std::path::PathBuf::from(&file_path);

    // Import the file
    let dest_path = state
        .downloader
        .import_local_file(&source_path)
        .await
        .map_err(|e| format!("Failed to import file: {}", e))?;

    // Determine media type from extension
    let extension = source_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let media_type = match extension.as_str() {
        "gif" => MediaType::Gif,
        "png" | "jpg" | "jpeg" | "webp" => MediaType::Image,
        "mp4" | "webm" | "mov" => MediaType::Video,
        _ => MediaType::Gif,
    };

    // Get file metadata
    let filename = dest_path.file_name().unwrap().to_string_lossy().to_string();

    let file_size = Downloader::get_file_size(&dest_path)
        .await
        .ok()
        .map(|s| s as i64);

    // Try to get image dimensions
    let (width, height) = if let Ok(img) = image::open(&dest_path) {
        let (w, h) = img.dimensions();
        (Some(w as i32), Some(h as i32))
    } else {
        (None, None)
    };

    // Create favorite
    let mut favorite = Favorite::new(
        filename,
        Some(dest_path.to_string_lossy().to_string()),
        media_type,
    );

    if let (Some(w), Some(h)) = (width, height) {
        favorite = favorite.with_dimensions(w, h);
    }

    favorite.file_size = file_size;

    // Save to database
    let favorites_db = FavoritesDb::new(state.db.pool());
    let id = favorites_db
        .create(&favorite)
        .await
        .map_err(|e| format!("Failed to save favorite: {}", e))?;

    favorite.id = Some(id);

    Ok(favorite)
}

#[tauri::command]
pub async fn add_klipy_favorite(
    gif_url: String,
    mp4_url: Option<String>,
    source_id: String,
    source_url: String,
    title: String,
    width: i32,
    height: i32,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Favorite, String> {
    let state = state.lock().await;

    // Download both GIF (for clipboard) and MP4 (for display)
    let (gif_path, mp4_path) = state
        .downloader
        .download_from_klipy(&gif_url, mp4_url.as_deref(), &source_id)
        .await
        .map_err(|e| format!("Failed to download media: {}", e))?;

    // Get file size of GIF
    let file_size = Downloader::get_file_size(&gif_path)
        .await
        .ok()
        .map(|s| s as i64);

    // Create favorite with both GIF and MP4 paths
    let mut favorite = Favorite::new(
        title.clone(),
        Some(gif_path.to_string_lossy().to_string()), // GIF for clipboard
        MediaType::Gif,
    )
    .with_gif_url(gif_url.clone()) // Keep URL as backup
    .with_dimensions(width, height)
    .with_source(Source::Klipy, Some(source_id), Some(source_url));

    favorite.mp4_filepath = mp4_path.map(|p| p.to_string_lossy().to_string()); // MP4 for display
    favorite.file_size = file_size;

    // Save to database
    let favorites_db = FavoritesDb::new(state.db.pool());
    let id = favorites_db
        .create(&favorite)
        .await
        .map_err(|e| format!("Failed to save favorite: {}", e))?;

    favorite.id = Some(id);

    Ok(favorite)
}
