use crate::commands::{AppState, CommandResult};
use crate::models::Favorite;
use crate::services::klipy::{KlipyAd, KlipyGif, KlipyItem};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub local: Vec<Favorite>,
    pub klipy: Option<KlipySearchResults>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KlipySearchResults {
    /// Mixed list of gifs and ads, in server-determined order. Frontend
    /// renders each kind with its own component.
    pub items: Vec<KlipyResultItem>,
    pub total_count: u32,
    pub page: u32,
}

/// Discriminated union over a `kind` tag — `kind` (not `type`) to avoid the
/// JS reserved-word issue and to match TS convention.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum KlipyResultItem {
    Gif(KlipyGifResult),
    Ad(KlipyAdResult),
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

/// Ad item passed through to the frontend. `content` is a full HTML document
/// to render in a sandboxed iframe; it handles its own click + impression
/// tracking. The width/height are intrinsic to the ad creative.
#[derive(Debug, Serialize, Deserialize)]
pub struct KlipyAdResult {
    pub width: u32,
    pub height: u32,
    pub content: String,
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

impl From<KlipyGif> for KlipyGifResult {
    fn from(gif: KlipyGif) -> Self {
        // HD is the original for download/clipboard; MD's dimensions match what we render.
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
    }
}

impl From<KlipyAd> for KlipyAdResult {
    fn from(ad: KlipyAd) -> Self {
        KlipyAdResult {
            width: ad.width,
            height: ad.height,
            content: ad.content,
        }
    }
}

fn map_items(items: Vec<KlipyItem>) -> Vec<KlipyResultItem> {
    items
        .into_iter()
        .filter_map(|item| match item {
            KlipyItem::Gif(gif) => Some(KlipyResultItem::Gif(gif.into())),
            KlipyItem::Ad(ad) => Some(KlipyResultItem::Ad(ad.into())),
            KlipyItem::Unknown => None,
        })
        .collect()
}

#[tauri::command]
pub async fn search_local(
    query: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult<Vec<Favorite>> {
    Ok(state.db.favorites().search(&query).await?)
}

#[tauri::command]
pub async fn search_klipy(
    query: String,
    limit: u32,
    page: u32,
    show_ads: bool,
    state: tauri::State<'_, AppState>,
) -> CommandResult<KlipySearchResults> {
    let response = state
        .klipy(show_ads)
        .search(&query, limit, page, state.ad_context_for(show_ads))
        .await?;

    Ok(KlipySearchResults {
        items: map_items(response.data.data),
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
    state: tauri::State<'_, AppState>,
) -> CommandResult<SearchResult> {
    let local = state.db.favorites().search(&query).await?;
    let klipy = search_klipy(query, klipy_limit, klipy_page, show_ads, state)
        .await
        .ok();
    Ok(SearchResult { local, klipy })
}

#[tauri::command]
pub async fn get_klipy_trending(
    limit: u32,
    page: u32,
    show_ads: bool,
    state: tauri::State<'_, AppState>,
) -> CommandResult<KlipySearchResults> {
    let response = state
        .klipy(show_ads)
        .trending(limit, page, state.ad_context_for(show_ads))
        .await?;
    Ok(KlipySearchResults {
        items: map_items(response.data.data),
        total_count: response.data.total.unwrap_or(0),
        page: response.data.current_page.unwrap_or(page),
    })
}

#[tauri::command]
pub async fn get_klipy_categories(
    show_ads: bool,
    state: tauri::State<'_, AppState>,
) -> CommandResult<KlipyCategoriesResult> {
    let response = state.klipy(show_ads).categories().await?;

    let categories = response
        .data
        .categories
        .into_iter()
        .map(|cat| KlipyCategory {
            name: cat.category,
            slug: cat.query,
            gif_url: cat.preview_url,
            mp4_url: None,
            // Category previews don't expose dimensions; pick a reasonable square
            // so the masonry layout can lay them out without flicker.
            width: 200,
            height: 200,
        })
        .collect();

    Ok(KlipyCategoriesResult { categories })
}

#[tauri::command]
pub async fn get_autocomplete(
    query: String,
    limit: u32,
    show_ads: bool,
    state: tauri::State<'_, AppState>,
) -> CommandResult<Vec<String>> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    Ok(state.klipy(show_ads).autocomplete(&query, limit).await?)
}

#[tauri::command]
pub async fn get_search_suggestions(
    query: String,
    limit: u32,
    show_ads: bool,
    state: tauri::State<'_, AppState>,
) -> CommandResult<Vec<String>> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    Ok(state
        .klipy(show_ads)
        .search_suggestions(&query, limit)
        .await?)
}

#[tauri::command]
pub async fn download_gif_temp(
    gif_url: String,
    filename: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult<String> {
    let file_path = state.downloader.download_temp(&gif_url, &filename).await?;
    Ok(file_path.to_string_lossy().into_owned())
}
