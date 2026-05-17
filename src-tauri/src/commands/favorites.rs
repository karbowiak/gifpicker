use crate::commands::{AppState, CommandError, CommandResult};
use crate::models::{Favorite, MediaType, Source};
use crate::services::Downloader;
use image::GenericImageView;

#[tauri::command]
pub async fn get_all_favorites(state: tauri::State<'_, AppState>) -> CommandResult<Vec<Favorite>> {
    Ok(state.db.favorites().get_all().await?)
}

#[tauri::command]
pub async fn get_favorite_by_id(
    id: i64,
    state: tauri::State<'_, AppState>,
) -> CommandResult<Option<Favorite>> {
    Ok(state.db.favorites().get_by_id(id).await?)
}

#[tauri::command]
pub async fn add_favorite(
    favorite: Favorite,
    state: tauri::State<'_, AppState>,
) -> CommandResult<i64> {
    Ok(state.db.favorites().create(&favorite).await?)
}

#[tauri::command]
pub async fn update_favorite(
    favorite: Favorite,
    state: tauri::State<'_, AppState>,
) -> CommandResult<()> {
    state.db.favorites().update(&favorite).await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_favorite(id: i64, state: tauri::State<'_, AppState>) -> CommandResult<()> {
    let favorites = state.db.favorites();

    // Best-effort: delete files we own before removing the row.
    if let Some(favorite) = favorites.get_by_id(id).await? {
        for path in [favorite.filepath.as_deref(), favorite.mp4_filepath.as_deref()]
            .into_iter()
            .flatten()
        {
            let path = std::path::PathBuf::from(path);
            if path.exists() {
                Downloader::delete_file(&path).await?;
            }
        }
    }

    favorites.delete(id).await?;
    Ok(())
}

#[tauri::command]
pub async fn increment_use_count(
    id: i64,
    state: tauri::State<'_, AppState>,
) -> CommandResult<()> {
    state.db.favorites().increment_use_count(id).await?;
    Ok(())
}

#[tauri::command]
pub async fn import_local_file(
    file_path: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult<Favorite> {
    let source_path = std::path::PathBuf::from(&file_path);

    let dest_path = state.downloader.import_local_file(&source_path).await?;

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

    let filename = dest_path
        .file_name()
        .ok_or_else(|| CommandError::InvalidArgument("imported file has no name".into()))?
        .to_string_lossy()
        .into_owned();

    let file_size = Downloader::get_file_size(&dest_path)
        .await
        .ok()
        .map(|s| s as i64);

    let dimensions = image::open(&dest_path)
        .ok()
        .map(|img| img.dimensions());

    let mut favorite = Favorite::new(
        filename,
        Some(dest_path.to_string_lossy().into_owned()),
        media_type,
    );

    if let Some((w, h)) = dimensions {
        favorite = favorite.with_dimensions(w as i32, h as i32);
    }
    favorite.file_size = file_size;

    let id = state.db.favorites().create(&favorite).await?;
    favorite.id = Some(id);
    Ok(favorite)
}

/// Download a Klipy GIF (and its MP4 preview, if any) and persist it as a favorite.
///
/// Replaces the old `add_klipy_favorite` + `download_klipy_gif` pair, which
/// were near-duplicates with different argument shapes.
#[tauri::command]
pub async fn add_klipy_favorite(
    gif_url: String,
    mp4_url: Option<String>,
    source_id: String,
    source_url: Option<String>,
    title: String,
    width: i32,
    height: i32,
    state: tauri::State<'_, AppState>,
) -> CommandResult<Favorite> {
    let (gif_path, mp4_path) = state
        .downloader
        .download_from_klipy(&gif_url, mp4_url.as_deref(), &source_id)
        .await?;

    let filename = gif_path
        .file_name()
        .ok_or_else(|| CommandError::InvalidArgument("downloaded file has no name".into()))?
        .to_string_lossy()
        .into_owned();

    let file_size = Downloader::get_file_size(&gif_path)
        .await
        .ok()
        .map(|s| s as i64);

    let mut favorite = Favorite::new(
        filename,
        Some(gif_path.to_string_lossy().into_owned()),
        MediaType::Gif,
    )
    .with_gif_url(gif_url)
    .with_dimensions(width, height)
    .with_source(Source::Klipy, Some(source_id), source_url);

    favorite.mp4_filepath = mp4_path.map(|p| p.to_string_lossy().into_owned());
    favorite.file_size = file_size;
    favorite.description = Some(title);

    let id = state.db.favorites().create(&favorite).await?;
    favorite.id = Some(id);
    Ok(favorite)
}
