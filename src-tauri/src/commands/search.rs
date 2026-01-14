use crate::commands::AppState;
use crate::config::{KLIPY_API_KEY_NO_ADS, KLIPY_API_KEY_WITH_ADS};
use crate::db::FavoritesDb;
use crate::models::Favorite;
use crate::services::KlipyClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub local: Vec<Favorite>,
    pub klipy: Option<KlipySearchResults>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KlipySearchResults {
    pub gifs: Vec<KlipyGifResult>,
    pub total_count: u32,
    pub page: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KlipyGifResult {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub url: String,
    pub gif_url: String,
    pub mp4_url: Option<String>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KlipyCategory {
    pub name: String,
    pub slug: String,
    pub gif_url: String,
    pub mp4_url: Option<String>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KlipyCategoriesResult {
    pub categories: Vec<KlipyCategory>,
}

/// Get the appropriate API key based on ads setting
fn get_api_key(show_ads: bool) -> String {
    if show_ads {
        KLIPY_API_KEY_WITH_ADS.to_string()
    } else {
        KLIPY_API_KEY_NO_ADS.to_string()
    }
}

#[tauri::command]
pub async fn search_local(
    query: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<Favorite>, String> {
    let state = state.lock().await;
    let favorites_db = FavoritesDb::new(state.db.pool());

    favorites_db
        .search(&query)
        .await
        .map_err(|e| format!("Failed to search favorites: {}", e))
}

#[tauri::command]
pub async fn search_klipy(
    query: String,
    limit: u32,
    page: u32,
    show_ads: bool,
) -> Result<KlipySearchResults, String> {
    let api_key = get_api_key(show_ads);
    let client = KlipyClient::new(api_key);

    let response = client
        .search(&query, limit, page)
        .await
        .map_err(|e| format!("Failed to search Klipy: {}", e))?;

    let gifs = response
        .data
        .data
        .into_iter()
        .map(|gif| {
            // Use HD format for original GIF, MD for display
            let hd = &gif.file.hd;
            let md = &gif.file.md;

            KlipyGifResult {
                id: gif.id.to_string(),
                slug: gif.slug.clone(),
                title: gif.title,
                url: format!("https://klipy.com/gifs/{}", gif.slug),
                gif_url: hd.gif.url.clone(),
                mp4_url: hd.mp4.as_ref().map(|m| m.url.clone()),
                width: md.gif.width,
                height: md.gif.height,
            }
        })
        .collect();

    Ok(KlipySearchResults {
        gifs,
        total_count: response.data.total.unwrap_or(0),
        page: response.data.current_page.unwrap_or(page),
    })
}

#[tauri::command]
pub async fn search_combined(
    query: String,
    klipy_limit: u32,
    klipy_page: u32,
    show_ads: bool,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<SearchResult, String> {
    // Search local favorites
    let local = search_local(query.clone(), state).await?;

    // Search Klipy
    let klipy = search_klipy(query, klipy_limit, klipy_page, show_ads)
        .await
        .ok();

    Ok(SearchResult { local, klipy })
}

#[tauri::command]
pub async fn get_klipy_trending(
    limit: u32,
    page: u32,
    show_ads: bool,
) -> Result<KlipySearchResults, String> {
    let api_key = get_api_key(show_ads);
    let client = KlipyClient::new(api_key);

    let response = client
        .trending(limit, page)
        .await
        .map_err(|e| format!("Failed to get trending GIFs: {}", e))?;

    let gifs = response
        .data
        .data
        .into_iter()
        .map(|gif| {
            let hd = &gif.file.hd;
            let md = &gif.file.md;

            KlipyGifResult {
                id: gif.id.to_string(),
                slug: gif.slug.clone(),
                title: gif.title,
                url: format!("https://klipy.com/gifs/{}", gif.slug),
                gif_url: hd.gif.url.clone(),
                mp4_url: hd.mp4.as_ref().map(|m| m.url.clone()),
                width: md.gif.width,
                height: md.gif.height,
            }
        })
        .collect();

    Ok(KlipySearchResults {
        gifs,
        total_count: response.data.total.unwrap_or(0),
        page: response.data.current_page.unwrap_or(page),
    })
}

#[tauri::command]
pub async fn get_klipy_categories(
    show_ads: bool,
) -> Result<KlipyCategoriesResult, String> {
    let api_key = get_api_key(show_ads);
    let client = KlipyClient::new(api_key);

    let response = client
        .categories()
        .await
        .map_err(|e| format!("Failed to get categories: {}", e))?;

    let categories = response
        .data
        .categories
        .into_iter()
        .map(|cat| {
            KlipyCategory {
                name: cat.category,
                slug: cat.query.clone(),
                gif_url: cat.preview_url,
                mp4_url: None, // Categories only have GIF preview
                width: 200,    // Default dimensions for preview
                height: 200,
            }
        })
        .collect();

    Ok(KlipyCategoriesResult { categories })
}

#[tauri::command]
pub async fn get_autocomplete(
    query: String,
    limit: u32,
    show_ads: bool,
) -> Result<Vec<String>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let api_key = get_api_key(show_ads);
    let client = KlipyClient::new(api_key);

    client
        .autocomplete(&query, limit)
        .await
        .map_err(|e| format!("Failed to get autocomplete: {}", e))
}

#[tauri::command]
pub async fn get_search_suggestions(
    query: String,
    limit: u32,
    show_ads: bool,
) -> Result<Vec<String>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let api_key = get_api_key(show_ads);
    let client = KlipyClient::new(api_key);

    client
        .search_suggestions(&query, limit)
        .await
        .map_err(|e| format!("Failed to get search suggestions: {}", e))
}

#[tauri::command]
pub async fn download_klipy_gif(
    klipy_slug: String,
    gif_url: String,
    mp4_url: Option<String>,
    title: String,
    width: u32,
    height: u32,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Favorite, String> {
    let state = state.lock().await;

    // Download both GIF and MP4
    let (gif_path, mp4_path) = state
        .downloader
        .download_from_klipy(&gif_url, mp4_url.as_deref(), &klipy_slug)
        .await
        .map_err(|e| format!("Failed to download media: {}", e))?;

    let filename = gif_path.file_name().unwrap().to_string_lossy().to_string();

    let file_size = crate::services::Downloader::get_file_size(&gif_path)
        .await
        .ok()
        .map(|s| s as i64);

    // Create favorite
    let mut favorite = crate::models::Favorite::new(
        filename,
        Some(gif_path.to_string_lossy().to_string()),
        crate::models::MediaType::Gif,
    )
    .with_source(
        crate::models::Source::Klipy,
        Some(klipy_slug),
        Some(gif_url),
    )
    .with_dimensions(width as i32, height as i32);

    favorite.mp4_filepath = mp4_path.map(|p| p.to_string_lossy().to_string());
    favorite.file_size = file_size;
    favorite.description = Some(title);

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
pub async fn download_gif_temp(
    gif_url: String,
    filename: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let state = state.lock().await;

    // Download the GIF to a temporary location
    let file_path = state
        .downloader
        .download_temp(&gif_url, &filename)
        .await
        .map_err(|e| format!("Failed to download GIF: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}
