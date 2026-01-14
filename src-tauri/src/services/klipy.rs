use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const KLIPY_API_BASE_URL: &str = "https://api.klipy.co/api/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlipyGif {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub file: KlipyFileFormats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlipyFileFormats {
    pub hd: KlipySizeFormat,
    pub md: KlipySizeFormat,
    pub sm: KlipySizeFormat,
    pub xs: KlipySizeFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlipySizeFormat {
    pub gif: KlipyMediaFile,
    pub webp: Option<KlipyMediaFile>,
    pub mp4: Option<KlipyMediaFile>,
    pub webm: Option<KlipyMediaFile>,
    pub jpg: Option<KlipyMediaFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlipyMediaFile {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlipySearchResponse {
    pub result: bool,
    pub data: KlipySearchData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlipySearchData {
    pub data: Vec<KlipyGif>,
    pub current_page: Option<u32>,
    pub last_page: Option<u32>,
    pub per_page: Option<u32>,
    pub total: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlipyCategoriesResponse {
    pub result: bool,
    pub data: KlipyCategoriesData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlipyCategoriesData {
    pub locale: String,
    pub categories: Vec<KlipyCategoryData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlipyCategoryData {
    pub category: String,
    pub query: String,
    pub preview_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlipyStringListResponse {
    pub result: bool,
    pub data: Vec<String>,
}

pub struct KlipyClient {
    client: Client,
    api_key: String,
}

impl KlipyClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Search for GIFs on Klipy
    pub async fn search(
        &self,
        query: &str,
        per_page: u32,
        page: u32,
    ) -> Result<KlipySearchResponse> {
        let url = format!("{}/{}/gifs/search", KLIPY_API_BASE_URL, self.api_key);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("q", query),
                ("per_page", &per_page.to_string()),
                ("page", &page.to_string()),
            ])
            .send()
            .await
            .context("Failed to send request to Klipy API")?;

        if !response.status().is_success() {
            anyhow::bail!("Klipy API returned error status: {}", response.status());
        }

        let search_response = response
            .json::<KlipySearchResponse>()
            .await
            .context("Failed to parse Klipy API response")?;

        Ok(search_response)
    }

    /// Get trending GIFs
    pub async fn trending(&self, per_page: u32, page: u32) -> Result<KlipySearchResponse> {
        let url = format!("{}/{}/gifs/trending", KLIPY_API_BASE_URL, self.api_key);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("per_page", &per_page.to_string()),
                ("page", &page.to_string()),
            ])
            .send()
            .await
            .context("Failed to send request to Klipy API")?;

        if !response.status().is_success() {
            anyhow::bail!("Klipy API returned error status: {}", response.status());
        }

        let search_response = response
            .json::<KlipySearchResponse>()
            .await
            .context("Failed to parse Klipy API response")?;

        Ok(search_response)
    }

    /// Get a GIF by slug
    pub async fn get_by_slug(&self, slug: &str) -> Result<KlipyGif> {
        let url = format!("{}/{}/gifs/items", KLIPY_API_BASE_URL, self.api_key);

        let response = self
            .client
            .get(&url)
            .query(&[("slugs", slug)])
            .send()
            .await
            .context("Failed to send request to Klipy API")?;

        if !response.status().is_success() {
            anyhow::bail!("Klipy API returned error status: {}", response.status());
        }

        let search_response = response
            .json::<KlipySearchResponse>()
            .await
            .context("Failed to parse Klipy API response")?;

        search_response
            .data
            .data
            .into_iter()
            .next()
            .context("GIF not found")
    }

    /// Get GIF categories
    pub async fn categories(&self) -> Result<KlipyCategoriesResponse> {
        let url = format!("{}/{}/gifs/categories", KLIPY_API_BASE_URL, self.api_key);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send request to Klipy API")?;

        if !response.status().is_success() {
            anyhow::bail!("Klipy API returned error status: {}", response.status());
        }

        let categories_response = response
            .json::<KlipyCategoriesResponse>()
            .await
            .context("Failed to parse Klipy API categories response")?;

        Ok(categories_response)
    }

    /// Get autocomplete suggestions for a query
    pub async fn autocomplete(&self, query: &str, limit: u32) -> Result<Vec<String>> {
        let url = format!(
            "{}/{}/autocomplete/{}",
            KLIPY_API_BASE_URL, self.api_key, query
        );

        let response = self
            .client
            .get(&url)
            .query(&[("limit", &limit.to_string())])
            .send()
            .await
            .context("Failed to send request to Klipy API")?;

        if !response.status().is_success() {
            anyhow::bail!("Klipy API returned error status: {}", response.status());
        }

        let result: KlipyStringListResponse = response
            .json()
            .await
            .context("Failed to parse Klipy API autocomplete response")?;

        Ok(result.data)
    }

    /// Get search suggestions for a query
    pub async fn search_suggestions(&self, query: &str, limit: u32) -> Result<Vec<String>> {
        let url = format!(
            "{}/{}/search-suggestions/{}",
            KLIPY_API_BASE_URL, self.api_key, query
        );

        let response = self
            .client
            .get(&url)
            .query(&[("limit", &limit.to_string())])
            .send()
            .await
            .context("Failed to send request to Klipy API")?;

        if !response.status().is_success() {
            anyhow::bail!("Klipy API returned error status: {}", response.status());
        }

        let result: KlipyStringListResponse = response
            .json()
            .await
            .context("Failed to parse Klipy API search suggestions response")?;

        Ok(result.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_klipy_client_creation() {
        let client = KlipyClient::new("test_key".to_string());
        assert_eq!(client.api_key, "test_key");
    }

    #[test]
    fn test_klipy_response_deserialization() {
        let json = r#"{
            "result": true,
            "data": {
                "data": [{
                    "id": 8679151651012575,
                    "slug": "happy-cat-f7u",
                    "title": "Happy Cat",
                    "file": {
                        "hd": {
                            "gif": {"url": "https://static.klipy.com/test.gif", "width": 480, "height": 270, "size": 1234567},
                            "webp": {"url": "https://static.klipy.com/test.webp", "width": 480, "height": 270, "size": 123456},
                            "mp4": {"url": "https://static.klipy.com/test.mp4", "width": 480, "height": 270, "size": 12345},
                            "webm": {"url": "https://static.klipy.com/test.webm", "width": 480, "height": 270, "size": 12345},
                            "jpg": {"url": "https://static.klipy.com/test.jpg", "width": 480, "height": 270, "size": 1234}
                        },
                        "md": {
                            "gif": {"url": "https://static.klipy.com/test_md.gif", "width": 320, "height": 180}
                        },
                        "sm": {
                            "gif": {"url": "https://static.klipy.com/test_sm.gif", "width": 200, "height": 113}
                        },
                        "xs": {
                            "gif": {"url": "https://static.klipy.com/test_xs.gif", "width": 100, "height": 56}
                        }
                    }
                }],
                "current_page": 1,
                "last_page": 10,
                "per_page": 25,
                "total": 250
            }
        }"#;

        let response: Result<KlipySearchResponse, _> = serde_json::from_str(json);
        assert!(response.is_ok());

        let response = response.unwrap();
        assert!(response.result);
        assert_eq!(response.data.data.len(), 1);
        assert_eq!(response.data.data[0].slug, "happy-cat-f7u");
        assert_eq!(response.data.data[0].title, "Happy Cat");
    }
}
