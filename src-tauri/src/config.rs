//! Application configuration constants
//!
//! This file contains API keys and other configuration values that are
//! embedded into the application at compile time.

/// Klipy API App Key (with ads enabled - supports development)
///
/// This key is used for accessing the Klipy GIF API with ads.
/// Get your own key at: https://klipy.com/developers
pub const KLIPY_API_KEY_WITH_ADS: &str =
    "Piv47mE6RutGKDIu1VkoqNsIUNKpaEa6cEhzAZWP3gPgfSJXpYYfWzLhsEiVZ2Qv";

/// Klipy API App Key (without ads)
///
/// This key is used when the user has disabled ads in settings.
pub const KLIPY_API_KEY_NO_ADS: &str =
    "9CKW7ub0jWCyDC4TGI7IQKsE8TeUom0NzpflIfQuljAqx7WnayXzlWbBMOPYaAOx";
