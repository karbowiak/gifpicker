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

/// An inline ad item returned alongside content. The `content` field is a
/// self-contained HTML document that must be rendered in a WebView/iframe;
/// it handles its own impression and click tracking. See
/// https://docs.klipy.com/advertisements/displaying-an-ad.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlipyAd {
    pub width: u32,
    pub height: u32,
    pub content: String,
}

/// A single item in `data.data` — either a GIF or an ad. New variants Klipy
/// may add in the future are swallowed as `Unknown` so we never blow up the
/// whole search response on an unrecognized `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum KlipyItem {
    Gif(KlipyGif),
    Ad(KlipyAd),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlipySearchResponse {
    pub result: bool,
    pub data: KlipySearchData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlipySearchData {
    pub data: Vec<KlipyItem>,
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

/// Context passed on every ad-eligible request. None of these fields are
/// secrets — they're either app-level constants (os/make/app_version) or a
/// stable per-install UUID generated on first launch. See
/// https://docs.klipy.com/advertisements/receiving-an-ad for the parameter
/// reference; the four width/height bounds are *required* for ad delivery.
#[derive(Debug, Clone)]
pub struct AdContext {
    pub customer_id: String,
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32,
    pub os: &'static str,
    pub make: &'static str,
    pub app_version: &'static str,
}

impl AdContext {
    /// Returns query params, in the exact key shape Klipy expects (`ad-min-width`,
    /// `customer_id`, …).
    fn query_params(&self) -> Vec<(&'static str, String)> {
        vec![
            ("customer_id", self.customer_id.clone()),
            ("ad-min-width", self.min_width.to_string()),
            ("ad-max-width", self.max_width.to_string()),
            ("ad-min-height", self.min_height.to_string()),
            ("ad-max-height", self.max_height.to_string()),
            ("ad-os", self.os.to_string()),
            ("ad-make", self.make.to_string()),
            ("ad-app-version", self.app_version.to_string()),
        ]
    }
}

pub struct KlipyClient {
    client: Client,
    api_key: String,
}

impl KlipyClient {
    pub fn new(api_key: String, user_agent: &str) -> Self {
        // A real UA so Klipy's ad-fill targeting has something to work with.
        // Falls back to a default Client if for some reason builder fails — we'd
        // rather make requests without a custom UA than panic at startup.
        let client = Client::builder()
            .user_agent(user_agent)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client, api_key }
    }

    /// Search for GIFs on Klipy
    pub async fn search(
        &self,
        query: &str,
        per_page: u32,
        page: u32,
        ad_context: Option<&AdContext>,
    ) -> Result<KlipySearchResponse> {
        let url = format!("{}/{}/gifs/search", KLIPY_API_BASE_URL, self.api_key);

        let mut req = self.client.get(&url).query(&[
            ("q", query),
            ("per_page", &per_page.to_string()),
            ("page", &page.to_string()),
        ]);
        if let Some(ctx) = ad_context {
            req = req.query(&ctx.query_params());
        }

        let response = req
            .send()
            .await
            .context("Failed to send request to Klipy API")?;

        if !response.status().is_success() {
            anyhow::bail!("Klipy API returned error status: {}", response.status());
        }

        response
            .json::<KlipySearchResponse>()
            .await
            .context("Failed to parse Klipy API response")
    }

    /// Get trending GIFs
    pub async fn trending(
        &self,
        per_page: u32,
        page: u32,
        ad_context: Option<&AdContext>,
    ) -> Result<KlipySearchResponse> {
        let url = format!("{}/{}/gifs/trending", KLIPY_API_BASE_URL, self.api_key);

        let mut req = self
            .client
            .get(&url)
            .query(&[("per_page", &per_page.to_string()), ("page", &page.to_string())]);
        if let Some(ctx) = ad_context {
            req = req.query(&ctx.query_params());
        }

        let response = req
            .send()
            .await
            .context("Failed to send request to Klipy API")?;

        if !response.status().is_success() {
            anyhow::bail!("Klipy API returned error status: {}", response.status());
        }

        response
            .json::<KlipySearchResponse>()
            .await
            .context("Failed to parse Klipy API response")
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
            .find_map(|item| match item {
                KlipyItem::Gif(gif) => Some(gif),
                _ => None,
            })
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

        response
            .json::<KlipyCategoriesResponse>()
            .await
            .context("Failed to parse Klipy API categories response")
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
        let client = KlipyClient::new("test_key".to_string(), "test-ua/1.0");
        assert_eq!(client.api_key, "test_key");
    }

    #[test]
    fn test_klipy_response_deserialization() {
        let json = r#"{
            "result": true,
            "data": {
                "data": [{
                    "type": "gif",
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
                        "md": { "gif": {"url": "https://static.klipy.com/test_md.gif", "width": 320, "height": 180} },
                        "sm": { "gif": {"url": "https://static.klipy.com/test_sm.gif", "width": 200, "height": 113} },
                        "xs": { "gif": {"url": "https://static.klipy.com/test_xs.gif", "width": 100, "height": 56} }
                    }
                }],
                "current_page": 1,
                "last_page": 10,
                "per_page": 25,
                "total": 250
            }
        }"#;

        let response: KlipySearchResponse = serde_json::from_str(json).unwrap();
        assert!(response.result);
        assert_eq!(response.data.data.len(), 1);
        match &response.data.data[0] {
            KlipyItem::Gif(g) => {
                assert_eq!(g.slug, "happy-cat-f7u");
                assert_eq!(g.title, "Happy Cat");
            }
            other => panic!("expected Gif, got {:?}", other),
        }
    }

    #[test]
    fn test_ad_item_deserializes_alongside_gifs() {
        // Real shape returned by Klipy when ads are enabled — see audit run on
        // 2026-05-17 in the project history.
        let json = r#"{
            "result": true,
            "data": {
                "data": [
                    { "type": "ad", "width": 320, "height": 100, "content": "<html>...</html>" },
                    { "type": "gif", "id": 1, "slug": "x", "title": "y",
                      "file": {
                          "hd": { "gif": {"url":"u","width":1,"height":1} },
                          "md": { "gif": {"url":"u","width":1,"height":1} },
                          "sm": { "gif": {"url":"u","width":1,"height":1} },
                          "xs": { "gif": {"url":"u","width":1,"height":1} }
                      }
                    },
                    { "type": "sticker", "id": 2 }
                ]
            }
        }"#;

        let response: KlipySearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.data.len(), 3);
        assert!(matches!(response.data.data[0], KlipyItem::Ad(_)));
        assert!(matches!(response.data.data[1], KlipyItem::Gif(_)));
        // Unknown types (e.g. future "sticker") are tolerated, not fatal.
        assert!(matches!(response.data.data[2], KlipyItem::Unknown));
    }
}
