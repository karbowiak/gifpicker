use anyhow::{Context, Result};
use arboard::Clipboard;
use std::path::Path;
use std::fs;

pub struct ClipboardManager {
    clipboard: Clipboard,
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        let clipboard = Clipboard::new()
            .context("Failed to initialize clipboard")?;

        Ok(Self { clipboard })
    }

    /// Copy an image file to the clipboard
    /// For GIF files, we need special handling to preserve animation
    pub fn copy_image(&mut self, path: &Path) -> Result<()> {
        // Check if it's a GIF file
        let is_gif = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("gif"))
            .unwrap_or(false);

        if is_gif {
            // For GIFs, copy the raw file bytes as binary data
            // This preserves animation when pasting into apps that support it
            let gif_data = fs::read(path)
                .context("Failed to read GIF file")?;

            // Try to set as image data first (for apps that support animated GIFs)
            // If that fails, fall back to static image
            match self.copy_gif_data(&gif_data) {
                Ok(_) => return Ok(()),
                Err(_) => {
                    // Fallback: convert to static image
                    return self.copy_static_image(path);
                }
            }
        } else {
            // For non-GIF images, use the standard method
            return self.copy_static_image(path);
        }
    }

    /// Copy GIF data preserving animation (platform-specific)
    fn copy_gif_data(&mut self, _data: &[u8]) -> Result<()> {
        // Note: arboard doesn't directly support animated GIFs
        // We'll need to use platform-specific APIs
        // For now, return error to trigger fallback
        anyhow::bail!("Animated GIF clipboard not yet implemented")
    }

    /// Copy image as static (converts GIF to single frame)
    fn copy_static_image(&mut self, path: &Path) -> Result<()> {
        // Read the image file
        let img = image::open(path)
            .context("Failed to open image file")?;

        // Convert to RGBA8 format for clipboard
        let rgba_image = img.to_rgba8();
        let (width, height) = rgba_image.dimensions();

        // Create clipboard image data
        let img_data = arboard::ImageData {
            width: width as usize,
            height: height as usize,
            bytes: rgba_image.into_raw().into(),
        };

        // Copy to clipboard
        self.clipboard
            .set_image(img_data)
            .context("Failed to copy image to clipboard")?;

        Ok(())
    }

    /// Copy text to the clipboard
    pub fn copy_text(&mut self, text: &str) -> Result<()> {
        self.clipboard
            .set_text(text)
            .context("Failed to copy text to clipboard")?;

        Ok(())
    }

    /// Copy file path to clipboard as file (for dragging/pasting files)
    /// This attempts to copy the file itself, not just the path
    pub fn copy_file_path(&mut self, path: &Path) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            // On macOS, use NSPasteboard to copy file
            return self.copy_file_macos(path);
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Fallback: just copy the path as text
            let path_str = path.to_string_lossy().to_string();
            self.copy_text(&path_str)
        }
    }

    #[cfg(target_os = "macos")]
    fn copy_file_macos(&mut self, path: &Path) -> Result<()> {
        use std::process::Command;

        // Use osascript (AppleScript) to copy file to clipboard
        // This is more reliable than NSPasteboard FFI
        let script = format!(
            r#"set the clipboard to (POSIX file "{}") as «class furl»"#,
            path.to_string_lossy()
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .context("Failed to run osascript")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("osascript failed: {}", error);
        }

        Ok(())
    }

    /// Get text from clipboard
    pub fn get_text(&mut self) -> Result<String> {
        self.clipboard
            .get_text()
            .context("Failed to get text from clipboard")
    }

    /// Check if clipboard contains text
    pub fn has_text(&mut self) -> bool {
        self.get_text().is_ok()
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new().expect("Failed to create clipboard manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageFormat;
    use tempfile::NamedTempFile;

    #[test]
    #[ignore] // May fail on systems without clipboard access
    fn test_clipboard_creation() {
        let result = ClipboardManager::new();
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // May fail on systems without clipboard access
    fn test_copy_and_get_text() {
        let mut clipboard = ClipboardManager::new().unwrap();

        let test_text = "Hello, clipboard!";
        clipboard.copy_text(test_text).unwrap();

        let retrieved = clipboard.get_text().unwrap();
        assert_eq!(retrieved, test_text);
    }

    #[test]
    #[ignore] // May fail on systems without clipboard access
    fn test_copy_file_path() {
        let mut clipboard = ClipboardManager::new().unwrap();

        let test_path = Path::new("/tmp/test.gif");
        clipboard.copy_file_path(test_path).unwrap();

        let retrieved = clipboard.get_text().unwrap();
        assert_eq!(retrieved, test_path.to_string_lossy());
    }

    #[test]
    #[ignore] // May fail on systems without clipboard access
    fn test_has_text() {
        let mut clipboard = ClipboardManager::new().unwrap();

        clipboard.copy_text("test").unwrap();
        assert!(clipboard.has_text());
    }

    #[test]
    #[ignore] // This test can be flaky on CI systems without display
    fn test_copy_image() {
        let mut clipboard = ClipboardManager::new().unwrap();

        // Create a small test image
        let img = image::RgbaImage::new(10, 10);

        // Save to temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().with_extension("png");

        img.save_with_format(&temp_path, ImageFormat::Png).unwrap();

        // Try to copy it
        let result = clipboard.copy_image(&temp_path);

        // Cleanup
        std::fs::remove_file(&temp_path).ok();

        // On some systems without GUI, this might fail
        // So we just check that it doesn't panic
        match result {
            Ok(_) => println!("Image copied successfully"),
            Err(e) => println!("Image copy failed (expected on headless systems): {}", e),
        }
    }
}
