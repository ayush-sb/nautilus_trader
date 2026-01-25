//! Kite API credential storage and signing helpers.

#![allow(unused_assignments)]

use aws_lc_rs::digest::{Context, SHA256};
use hex;
use std::fmt::Debug;

use zeroize::ZeroizeOnDrop;

/// API credentials required for signing Kite requests.
#[derive(Clone, ZeroizeOnDrop)]
pub struct Credential {
    #[zeroize(skip)]
    api_key: String,
    #[zeroize(skip)]
    api_secret: String,
    #[zeroize(skip)]
    access_token: Option<String>,
}

impl Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credential")
            .field("api_key", &self.api_key)
            .field("api_secret", &"redacted")
            .finish()
    }
}

impl Credential {
    /// Creates a new [`Credential`] instance from the API key and secret.
    #[must_use]
    pub fn new(api_key: String, api_secret: String, access_token: Option<String>) -> Self {
        Self {
            api_key,
            api_secret,
            access_token,
        }
    }

    #[must_use]
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    #[must_use]
    pub fn api_key_masked(&self) -> String {
        nautilus_core::string::mask_api_key(self.api_key())
    }

    /// Generate Authorization header for Kite requests.
    /// "Authorization: token api_key:access_token"
    #[must_use]
    pub fn generate_auth_headers(&self) -> String {
        format!(
            "token {api_key}:{access_token}",
            api_key = self.api_key,
            access_token = self.access_token.clone().unwrap_or("".to_string()),
        )
    }

    /// Generate a checksum to be sent to API to generate the access_token
    /// The checksum is a SHA-256 hash of:
    /// (api_key + request_token + api_secret)
    #[must_use]
    pub fn generate_checksum(&self, request_token: String) -> String {
        let mut string = String::new();
        string.push_str(&self.api_key);
        string.push_str(&request_token);
        string.push_str(&self.api_secret);

        let mut context = Context::new(&SHA256);
        context.update(string.as_bytes());
        let result = context.finish();

        hex::encode(result.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    const API_KEY: &str = "test_api_key";
    const API_SECRET: &str = "test_secret";
    const ACCESS_TOKEN: &str = "test_access_token";

    #[rstest]
    fn generate_auth_headers_matches_reference() {
        let credential = Credential::new(
            API_KEY.to_string(),
            API_SECRET.to_string(),
            Some(ACCESS_TOKEN.to_string()),
        );
        let auth_headers = credential.generate_auth_headers();
        assert_eq!(auth_headers, "token test_api_key:test_access_token");
    }

    #[rstest]
    fn generate_checksum_matches_reference() {
        let credential = Credential::new(API_KEY.to_string(), API_SECRET.to_string(), None);
        let checksum = credential.generate_checksum("ABC123DEF456".to_string());
        assert_eq!(
            checksum,
            "cddc7dce709ad4bfe33ec01c53f09d8c3826e2221b4acc20a11093a65cfa2297"
        );
    }
}
