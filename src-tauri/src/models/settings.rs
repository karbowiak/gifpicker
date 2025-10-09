use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub giphy_api_key: Option<String>,
    pub hotkey: String,
    pub window_width: i32,
    pub window_height: i32,
    pub max_item_width: i32,
    pub close_after_selection: bool,
    pub launch_at_startup: bool,
    pub theme: Theme,
    pub clipboard_mode: ClipboardMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardMode {
    File,
    Url,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            giphy_api_key: None,
            #[cfg(target_os = "macos")]
            hotkey: "Option+Cmd+G".to_string(),
            #[cfg(not(target_os = "macos"))]
            hotkey: "Ctrl+Shift+G".to_string(),
            window_width: 800,
            window_height: 600,
            max_item_width: 400,
            close_after_selection: true,
            launch_at_startup: false,
            theme: Theme::System,
            clipboard_mode: ClipboardMode::File,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.window_width, 800);
        assert_eq!(settings.window_height, 600);
        assert!(settings.close_after_selection);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings.window_width, deserialized.window_width);
    }
}
