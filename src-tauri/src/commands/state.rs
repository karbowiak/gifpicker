use crate::config::{KLIPY_API_KEY_NO_ADS, KLIPY_API_KEY_WITH_ADS};
use crate::db::Database;
use crate::services::{AdContext, Downloader, KlipyClient};
use std::sync::Arc;

/// Bounds we tell Klipy our ad slot can fill. The lower bound is the smallest
/// banner Klipy serves; the upper bound is roughly one masonry column wide
/// (~400px) and a comfortable banner height (~250px). See
/// https://docs.klipy.com/advertisements/receiving-an-ad — these four bounds
/// are *required* for ad delivery to be eligible at all.
const AD_MIN_WIDTH: u32 = 50;
const AD_MAX_WIDTH: u32 = 400;
const AD_MIN_HEIGHT: u32 = 50;
const AD_MAX_HEIGHT: u32 = 250;

/// Sent to Klipy as the ad-targeting OS hint AND as part of the User-Agent
/// header. We don't fake mobile to inflate fill — desktop ad-fill is what it is.
#[cfg(target_os = "macos")]
const AD_OS: &str = "macos";
#[cfg(target_os = "windows")]
const AD_OS: &str = "windows";
#[cfg(target_os = "linux")]
const AD_OS: &str = "linux";
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
const AD_OS: &str = "other";

#[cfg(target_os = "macos")]
const AD_MAKE: &str = "apple";
#[cfg(not(target_os = "macos"))]
const AD_MAKE: &str = "pc";

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn user_agent() -> String {
    format!("GifPicker/{} ({})", APP_VERSION, AD_OS)
}

/// Shared application state.
///
/// Held by Tauri's state manager — `tauri::State<'_, AppState>` gives commands
/// shared access without any locking. All inner services are immutable, async-safe
/// handles (Arc-wrapped pools/clients), so concurrent commands run in parallel.
pub struct AppState {
    pub db: Arc<Database>,
    pub downloader: Arc<Downloader>,
    pub klipy_with_ads: Arc<KlipyClient>,
    pub klipy_no_ads: Arc<KlipyClient>,
    /// Sent on every ad-eligible Klipy call. Stable per install.
    pub ad_context: AdContext,
}

impl AppState {
    pub fn new(db: Arc<Database>, downloader: Arc<Downloader>, customer_id: String) -> Self {
        let ua = user_agent();
        Self {
            db,
            downloader,
            klipy_with_ads: Arc::new(KlipyClient::new(
                KLIPY_API_KEY_WITH_ADS.to_string(),
                &ua,
            )),
            klipy_no_ads: Arc::new(KlipyClient::new(KLIPY_API_KEY_NO_ADS.to_string(), &ua)),
            ad_context: AdContext {
                customer_id,
                min_width: AD_MIN_WIDTH,
                max_width: AD_MAX_WIDTH,
                min_height: AD_MIN_HEIGHT,
                max_height: AD_MAX_HEIGHT,
                os: AD_OS,
                make: AD_MAKE,
                app_version: APP_VERSION,
            },
        }
    }

    pub fn klipy(&self, show_ads: bool) -> &Arc<KlipyClient> {
        if show_ads {
            &self.klipy_with_ads
        } else {
            &self.klipy_no_ads
        }
    }

    /// Ad context to attach to a request — only when ads are wanted; otherwise
    /// None so we don't waste the round-trip param weight.
    pub fn ad_context_for(&self, show_ads: bool) -> Option<&AdContext> {
        show_ads.then_some(&self.ad_context)
    }
}
