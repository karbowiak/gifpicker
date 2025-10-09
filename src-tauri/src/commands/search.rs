use crate::commands::AppState;
use crate::db::FavoritesDb;
use crate::models::Favorite;
use crate::services::GiphyClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub local: Vec<Favorite>,
    pub giphy: Option<GiphySearchResults>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GiphySearchResults {
    pub gifs: Vec<GiphyGifResult>,
    pub total_count: u32,
    pub offset: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GiphyGifResult {
    pub id: String,
    pub title: String,
    pub url: String,
    pub gif_url: String,
    pub width: String,
    pub height: String,
}

#[tauri::command]
pub async fn search_local(
    query: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<Favorite>, String> {
    let state = state.lock().await;
    let favorites_db = FavoritesDb::new(state.db.pool());

    favorites_db.search(&query)
        .await
        .map_err(|e| format!("Failed to search favorites: {}", e))
}

#[tauri::command]
pub async fn search_giphy(
    query: String,
    limit: u32,
    offset: u32,
    api_key: String,
) -> Result<GiphySearchResults, String> {
    if api_key.is_empty() {
        return Err("Giphy API key not configured".to_string());
    }

    let client = GiphyClient::new(api_key);

    let response = client.search(&query, limit, offset)
        .await
        .map_err(|e| format!("Failed to search Giphy: {}", e))?;

    let gifs = response.data
        .into_iter()
        .map(|gif| GiphyGifResult {
            id: gif.id,
            title: gif.title,
            url: gif.url,
            // Use 'original' for actual GIF file, not 'downsized' which may return static image
            gif_url: gif.images.original.url,
            width: gif.images.original.width,
            height: gif.images.original.height,
        })
        .collect();

    Ok(GiphySearchResults {
        gifs,
        total_count: response.pagination.total_count,
        offset: response.pagination.offset,
    })
}

#[tauri::command]
pub async fn search_combined(
    query: String,
    giphy_limit: u32,
    giphy_offset: u32,
    api_key: Option<String>,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<SearchResult, String> {
    // Search local favorites
    let local = search_local(query.clone(), state).await?;

    // Search Giphy if API key is provided
    let giphy = if let Some(key) = api_key {
        if !key.is_empty() {
            search_giphy(query, giphy_limit, giphy_offset, key).await.ok()
        } else {
            None
        }
    } else {
        None
    };

    Ok(SearchResult { local, giphy })
}

#[tauri::command]
pub async fn get_giphy_trending(
    limit: u32,
    offset: u32,
    api_key: String,
) -> Result<GiphySearchResults, String> {
    if api_key.is_empty() {
        return Err("Giphy API key not configured".to_string());
    }

    let client = GiphyClient::new(api_key);

    let response = client.trending(limit, offset)
        .await
        .map_err(|e| format!("Failed to get trending GIFs: {}", e))?;

    let gifs = response.data
        .into_iter()
        .map(|gif| GiphyGifResult {
            id: gif.id,
            title: gif.title,
            url: gif.url,
            // Use 'original' for actual GIF file, not 'downsized' which may return static image
            gif_url: gif.images.original.url,
            width: gif.images.original.width,
            height: gif.images.original.height,
        })
        .collect();

    Ok(GiphySearchResults {
        gifs,
        total_count: response.pagination.total_count,
        offset: response.pagination.offset,
    })
}

#[tauri::command]
pub async fn download_giphy_gif(
    giphy_id: String,
    gif_url: String,
    title: String,
    width: String,
    height: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Favorite, String> {
    let state = state.lock().await;

    // Download the GIF
    let file_path = state.downloader.download_from_giphy(&gif_url, &giphy_id)
        .await
        .map_err(|e| format!("Failed to download GIF: {}", e))?;

    let filename = file_path.file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let file_size = crate::services::Downloader::get_file_size(&file_path)
        .await
        .ok()
        .map(|s| s as i64);

    // Parse dimensions
    let width_i32 = width.parse::<i32>().ok();
    let height_i32 = height.parse::<i32>().ok();

    // Create favorite
    let mut favorite = crate::models::Favorite::new(
        filename,
        Some(file_path.to_string_lossy().to_string()),
        crate::models::MediaType::Gif,
    )
    .with_source(
        crate::models::Source::Giphy,
        Some(giphy_id),
        Some(gif_url),
    );

    if let (Some(w), Some(h)) = (width_i32, height_i32) {
        favorite = favorite.with_dimensions(w, h);
    }

    favorite.file_size = file_size;
    favorite.description = Some(title);

    // Save to database
    let favorites_db = FavoritesDb::new(state.db.pool());
    let id = favorites_db.create(&favorite)
        .await
        .map_err(|e| format!("Failed to save favorite: {}", e))?;

    favorite.id = Some(id);

    Ok(favorite)
}

#[tauri::command]
pub async fn download_gif_temp(
    gif_url: String,
    filename: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let state = state.lock().await;

    // Download the GIF to a temporary location
    let file_path = state.downloader.download_temp(&gif_url, &filename)
        .await
        .map_err(|e| format!("Failed to download GIF: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}
