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
    Giphy,
    Tenor,
    Local,
    Upload,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Giphy => write!(f, "giphy"),
            Source::Tenor => write!(f, "tenor"),
            Source::Local => write!(f, "local"),
            Source::Upload => write!(f, "upload"),
        }
    }
}

impl std::str::FromStr for Source {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "giphy" => Ok(Source::Giphy),
            "tenor" => Ok(Source::Tenor),
            "local" => Ok(Source::Local),
            "upload" => Ok(Source::Upload),
            _ => Err(format!("Unknown source: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    pub id: Option<i64>,
    pub filename: String,
    pub filepath: Option<String>, // Made optional - not needed for Giphy GIFs
    pub gif_url: Option<String>, // Direct GIF URL for clipboard
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
    pub fn new(
        filename: String,
        filepath: Option<String>,
        media_type: MediaType,
    ) -> Self {
        Self {
            id: None,
            filename,
            filepath,
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

    pub fn with_source(mut self, source: Source, source_id: Option<String>, source_url: Option<String>) -> Self {
        self.source = Some(source);
        self.source_id = source_id;
        self.source_url = source_url;
        self
    }

    pub fn with_gif_url(mut self, gif_url: String) -> Self {
        self.gif_url = Some(gif_url);
        self
    }

    pub fn with_dimensions(mut self, width: i32, height: i32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_to_string() {
        assert_eq!(MediaType::Gif.to_string(), "gif");
        assert_eq!(MediaType::Image.to_string(), "image");
        assert_eq!(MediaType::Video.to_string(), "video");
    }

    #[test]
    fn test_media_type_from_string() {
        assert_eq!("gif".parse::<MediaType>().unwrap(), MediaType::Gif);
        assert_eq!("GIF".parse::<MediaType>().unwrap(), MediaType::Gif);
        assert_eq!("image".parse::<MediaType>().unwrap(), MediaType::Image);
        assert_eq!("video".parse::<MediaType>().unwrap(), MediaType::Video);
    }

    #[test]
    fn test_source_to_string() {
        assert_eq!(Source::Giphy.to_string(), "giphy");
        assert_eq!(Source::Local.to_string(), "local");
    }

    #[test]
    fn test_favorite_new() {
        let fav = Favorite::new(
            "test.gif".to_string(),
            Some("/path/to/test.gif".to_string()),
            MediaType::Gif,
        );

        assert_eq!(fav.filename, "test.gif");
        assert_eq!(fav.filepath, Some("/path/to/test.gif".to_string()));
        assert_eq!(fav.media_type, MediaType::Gif);
        assert_eq!(fav.use_count, 0);
        assert!(fav.id.is_none());
    }

    #[test]
    fn test_favorite_builder() {
        let fav = Favorite::new(
            "test.gif".to_string(),
            Some("/path/to/test.gif".to_string()),
            MediaType::Gif,
        )
        .with_source(Source::Giphy, Some("123".to_string()), Some("https://giphy.com/123".to_string()))
        .with_dimensions(500, 300)
        .with_tags(vec!["funny".to_string(), "cat".to_string()]);

        assert_eq!(fav.source, Some(Source::Giphy));
        assert_eq!(fav.source_id, Some("123".to_string()));
        assert_eq!(fav.width, Some(500));
        assert_eq!(fav.height, Some(300));
        assert_eq!(fav.tags.len(), 2);
    }
}
