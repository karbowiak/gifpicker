use anyhow::{Context, Result};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct Downloader {
    client: Client,
    media_dir: PathBuf,
}

impl Downloader {
    pub fn new(media_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            media_dir,
        })
    }

    /// Ensure media directory structure exists
    pub async fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.media_dir).await
            .context("Failed to create media directory")?;

        fs::create_dir_all(self.media_dir.join("gifs")).await
            .context("Failed to create gifs directory")?;

        fs::create_dir_all(self.media_dir.join("images")).await
            .context("Failed to create images directory")?;

        fs::create_dir_all(self.media_dir.join("videos")).await
            .context("Failed to create videos directory")?;

        Ok(())
    }

    /// Download a file from a URL and save it locally
    /// Returns the path where the file was saved
    pub async fn download(&self, url: &str, filename: &str, media_type: &str) -> Result<PathBuf> {
        self.ensure_directories().await?;

        // Create subdirectory path based on media type
        let subdir = match media_type {
            "gif" => "gifs",
            "image" => "images",
            "video" => "videos",
            _ => "gifs", // default to gifs
        };

        let file_path = self.media_dir.join(subdir).join(filename);

        // Check if file already exists
        if file_path.exists() {
            return Ok(file_path);
        }

        // Download the file
        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to download file")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download file: HTTP {}", response.status());
        }

        let bytes = response.bytes().await
            .context("Failed to read response body")?;

        // Write to file
        let mut file = fs::File::create(&file_path).await
            .context("Failed to create file")?;

        file.write_all(&bytes).await
            .context("Failed to write file")?;

        file.flush().await
            .context("Failed to flush file")?;

        Ok(file_path)
    }

    /// Download from Giphy with a generated filename based on the URL hash
    pub async fn download_from_giphy(&self, url: &str, giphy_id: &str) -> Result<PathBuf> {
        // Extract file extension from URL
        let extension = url.split('.').last().unwrap_or("gif");

        // Generate filename using giphy ID
        let filename = format!("giphy_{}.{}", giphy_id, extension);

        self.download(url, &filename, "gif").await
    }

    /// Copy a local file to the media directory
    pub async fn import_local_file(&self, source_path: &Path) -> Result<PathBuf> {
        self.ensure_directories().await?;

        let filename = source_path
            .file_name()
            .context("Invalid source file path")?
            .to_string_lossy()
            .to_string();

        // Determine media type from extension
        let extension = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let media_type = match extension.as_str() {
            "gif" => "gif",
            "png" | "jpg" | "jpeg" | "webp" => "image",
            "mp4" | "webm" | "mov" => "video",
            _ => "gif",
        };

        let subdir = match media_type {
            "gif" => "gifs",
            "image" => "images",
            "video" => "videos",
            _ => "gifs",
        };

        let dest_path = self.media_dir.join(subdir).join(&filename);

        // Copy the file
        fs::copy(source_path, &dest_path).await
            .context("Failed to copy file")?;

        Ok(dest_path)
    }

    /// Generate a unique filename based on content hash
    pub fn generate_hash_filename(content: &[u8], extension: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = hasher.finalize();
        format!("{:x}.{}", hash, extension)
    }

    /// Get the size of a file
    pub async fn get_file_size(path: &Path) -> Result<u64> {
        let metadata = fs::metadata(path).await
            .context("Failed to read file metadata")?;
        Ok(metadata.len())
    }

    /// Delete a file
    pub async fn delete_file(path: &Path) -> Result<()> {
        fs::remove_file(path).await
            .context("Failed to delete file")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_downloader() -> (Downloader, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let media_dir = temp_dir.path().join("media");
        let downloader = Downloader::new(media_dir).unwrap();
        (downloader, temp_dir)
    }

    #[tokio::test]
    async fn test_ensure_directories() {
        let (downloader, _temp) = create_test_downloader().await;

        downloader.ensure_directories().await.unwrap();

        assert!(downloader.media_dir.join("gifs").exists());
        assert!(downloader.media_dir.join("images").exists());
        assert!(downloader.media_dir.join("videos").exists());
    }

    #[tokio::test]
    #[ignore] // Requires internet connection
    async fn test_download_file() {
        let (downloader, _temp) = create_test_downloader().await;

        // Use a small test image from a reliable source
        let url = "https://via.placeholder.com/150.png";
        let filename = "test.png";

        let result = downloader.download(url, filename, "image").await;
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.exists());
        assert!(path.to_str().unwrap().contains("images"));
    }

    #[tokio::test]
    async fn test_import_local_file() {
        let (downloader, temp_dir) = create_test_downloader().await;

        // Create a test file
        let source_file = temp_dir.path().join("test.gif");
        tokio::fs::write(&source_file, b"test content").await.unwrap();

        let result = downloader.import_local_file(&source_file).await;
        assert!(result.is_ok());

        let dest_path = result.unwrap();
        assert!(dest_path.exists());
        assert!(dest_path.to_str().unwrap().contains("gifs"));

        let content = tokio::fs::read(&dest_path).await.unwrap();
        assert_eq!(content, b"test content");
    }

    #[test]
    fn test_generate_hash_filename() {
        let content = b"test content";
        let filename = Downloader::generate_hash_filename(content, "gif");

        assert!(filename.ends_with(".gif"));
        assert!(filename.len() > 10); // Hash should be long

        // Same content should generate same hash
        let filename2 = Downloader::generate_hash_filename(content, "gif");
        assert_eq!(filename, filename2);
    }

    #[tokio::test]
    async fn test_get_file_size() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        let test_content = b"Hello, World!";
        tokio::fs::write(&test_file, test_content).await.unwrap();

        let size = Downloader::get_file_size(&test_file).await.unwrap();
        assert_eq!(size, test_content.len() as u64);
    }

    #[tokio::test]
    async fn test_delete_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        tokio::fs::write(&test_file, b"test").await.unwrap();
        assert!(test_file.exists());

        Downloader::delete_file(&test_file).await.unwrap();
        assert!(!test_file.exists());
    }
}
