use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const GIPHY_API_BASE_URL: &str = "https://api.giphy.com/v1/gifs";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiphyGif {
    pub id: String,
    pub title: String,
    pub images: GiphyImages,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiphyImages {
    pub original: GiphyImage,
    pub fixed_width: GiphyImage,
    pub fixed_height: GiphyImage,
    pub downsized: GiphyImage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiphyImage {
    pub url: String,
    pub width: String,
    pub height: String,
    pub size: Option<String>,
    pub mp4: Option<String>,      // MP4 version for display (better performance)
    pub mp4_size: Option<String>, // Size of MP4 file
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiphySearchResponse {
    pub data: Vec<GiphyGif>,
    pub pagination: GiphyPagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiphyPagination {
    pub total_count: u32,
    pub count: u32,
    pub offset: u32,
}

pub struct GiphyClient {
    client: Client,
    api_key: String,
}

impl GiphyClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Search for GIFs on Giphy
    pub async fn search(&self, query: &str, limit: u32, offset: u32) -> Result<GiphySearchResponse> {
        let url = format!("{}/search", GIPHY_API_BASE_URL);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("api_key", self.api_key.as_str()),
                ("q", query),
                ("limit", &limit.to_string()),
                ("offset", &offset.to_string()),
                ("rating", "pg-13"),
                ("lang", "en"),
            ])
            .send()
            .await
            .context("Failed to send request to Giphy API")?;

        if !response.status().is_success() {
            anyhow::bail!("Giphy API returned error status: {}", response.status());
        }

        let search_response = response
            .json::<GiphySearchResponse>()
            .await
            .context("Failed to parse Giphy API response")?;

        Ok(search_response)
    }

    /// Get trending GIFs
    pub async fn trending(&self, limit: u32, offset: u32) -> Result<GiphySearchResponse> {
        let url = format!("{}/trending", GIPHY_API_BASE_URL);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("api_key", self.api_key.as_str()),
                ("limit", &limit.to_string()),
                ("offset", &offset.to_string()),
                ("rating", "pg-13"),
            ])
            .send()
            .await
            .context("Failed to send request to Giphy API")?;

        if !response.status().is_success() {
            anyhow::bail!("Giphy API returned error status: {}", response.status());
        }

        let search_response = response
            .json::<GiphySearchResponse>()
            .await
            .context("Failed to parse Giphy API response")?;

        Ok(search_response)
    }

    /// Get a GIF by ID
    pub async fn get_by_id(&self, id: &str) -> Result<GiphyGif> {
        let url = format!("{}/{}", GIPHY_API_BASE_URL, id);

        let response = self
            .client
            .get(&url)
            .query(&[("api_key", self.api_key.as_str())])
            .send()
            .await
            .context("Failed to send request to Giphy API")?;

        if !response.status().is_success() {
            anyhow::bail!("Giphy API returned error status: {}", response.status());
        }

        #[derive(Deserialize)]
        struct GiphyGetResponse {
            data: GiphyGif,
        }

        let gif_response = response
            .json::<GiphyGetResponse>()
            .await
            .context("Failed to parse Giphy API response")?;

        Ok(gif_response.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_api_key() -> String {
        // Giphy provides a public beta key for testing
        // You can use this or set GIPHY_API_KEY environment variable
        std::env::var("GIPHY_API_KEY")
            .unwrap_or_else(|_| "YOUR_API_KEY_HERE".to_string())
    }

    #[tokio::test]
    #[ignore] // Ignore by default since it requires internet and API key
    async fn test_search() {
        let api_key = get_test_api_key();
        if api_key == "YOUR_API_KEY_HERE" {
            println!("Skipping test - no API key provided");
            return;
        }

        let client = GiphyClient::new(api_key);
        let result = client.search("cat", 10, 0).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.data.is_empty());
        assert!(response.pagination.total_count > 0);
    }

    #[tokio::test]
    #[ignore] // Ignore by default since it requires internet and API key
    async fn test_trending() {
        let api_key = get_test_api_key();
        if api_key == "YOUR_API_KEY_HERE" {
            println!("Skipping test - no API key provided");
            return;
        }

        let client = GiphyClient::new(api_key);
        let result = client.trending(5, 0).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.len(), 5);
    }

    #[test]
    fn test_giphy_client_creation() {
        let client = GiphyClient::new("test_key".to_string());
        assert_eq!(client.api_key, "test_key");
    }

    #[test]
    fn test_giphy_response_deserialization() {
        let json = r#"{
            "data": [{
                "id": "test123",
                "title": "Test GIF",
                "url": "https://giphy.com/gifs/test123",
                "images": {
                    "original": {
                        "url": "https://media.giphy.com/media/test123/giphy.gif",
                        "width": "480",
                        "height": "270",
                        "size": "1234567"
                    },
                    "fixed_width": {
                        "url": "https://media.giphy.com/media/test123/200w.gif",
                        "width": "200",
                        "height": "113"
                    },
                    "fixed_height": {
                        "url": "https://media.giphy.com/media/test123/200h.gif",
                        "width": "355",
                        "height": "200"
                    },
                    "downsized": {
                        "url": "https://media.giphy.com/media/test123/giphy-downsized.gif",
                        "width": "400",
                        "height": "225"
                    }
                }
            }],
            "pagination": {
                "total_count": 1000,
                "count": 1,
                "offset": 0
            }
        }"#;

        let response: Result<GiphySearchResponse, _> = serde_json::from_str(json);
        assert!(response.is_ok());

        let response = response.unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].id, "test123");
        assert_eq!(response.pagination.total_count, 1000);
    }
}
