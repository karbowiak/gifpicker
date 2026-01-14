use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Gif,
    Image,
    Video,
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Gif => write!(f, "gif"),
            MediaType::Image => write!(f, "image"),
            MediaType::Video => write!(f, "video"),
        }
    }
}

impl std::str::FromStr for MediaType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "gif" => Ok(MediaType::Gif),
            "image" => Ok(MediaType::Image),
            "video" => Ok(MediaType::Video),
            _ => Err(format!("Unknown media type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Klipy,
    Local,
    Upload,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Klipy => write!(f, "klipy"),
            Source::Local => write!(f, "local"),
            Source::Upload => write!(f, "upload"),
        }
    }
}

impl std::str::FromStr for Source {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "klipy" => Ok(Source::Klipy),
            "local" => Ok(Source::Local),
            "upload" => Ok(Source::Upload),
            _ => Err(format!("Unknown source: {}", s)),
        }
    }
}

/// Favorite stores a GIF locally
/// - filepath: GIF file (for clipboard - Discord compatibility)
/// - mp4_filepath: MP4 file (for UI rendering - efficient)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    pub id: Option<i64>,
    pub filename: String,
    pub filepath: Option<String>,     // GIF file (for clipboard)
    pub mp4_filepath: Option<String>, // MP4 for UI display
    pub gif_url: Option<String>,      // Original URL as backup
    pub media_type: MediaType,
    pub source: Option<Source>,
    pub source_id: Option<String>,
    pub source_url: Option<String>,
    pub tags: Vec<String>,
    pub custom_tags: Vec<String>,
    pub description: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub file_size: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub use_count: i32,
}

impl Favorite {
    pub fn new(filename: String, filepath: Option<String>, media_type: MediaType) -> Self {
        Self {
            id: None,
            filename,
            filepath,
            mp4_filepath: None,
            gif_url: None,
            media_type,
            source: None,
            source_id: None,
            source_url: None,
            tags: Vec::new(),
            custom_tags: Vec::new(),
            description: None,
            width: None,
            height: None,
            file_size: None,
            created_at: Utc::now(),
            last_used: None,
            use_count: 0,
        }
    }

    pub fn with_source(
        mut self,
        source: Source,
        source_id: Option<String>,
        source_url: Option<String>,
    ) -> Self {
        self.source = Some(source);
        self.source_id = source_id;
        self.source_url = source_url;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_dimensions(mut self, width: i32, height: i32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn with_gif_url(mut self, url: String) -> Self {
        self.gif_url = Some(url);
        self
    }
}
