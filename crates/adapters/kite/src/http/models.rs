use serde::{Deserialize, Serialize};

pub trait KiteResponseCheck {
    fn status(&self) -> &str;
    fn message(&self) -> &str;
    fn error_type(&self) -> &str;
}

/// Generic wrapper that contains a list payload returned by Kite.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KiteList<T> {
    /// Collection returned by the endpoint.
    pub list: Vec<T>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KiteErrorCheck {
    pub status: String,
    pub message: String,
    pub error_type: String,
}

/// Top-level response envelope returned by Kite HTTP endpoints.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KiteResponse<T> {
    /// Status of the response ("success" or "error")
    pub status: String,
    /// Data, in case of successful response
    pub data: Option<T>,
    /// Error message, in case of error response
    pub message: Option<String>,
    pub error_type: Option<String>,
}

impl<T> KiteResponseCheck for KiteResponse<T> {
    fn status(&self) -> &str {
        &self.status
    }

    fn message(&self) -> &str {
        self.message.as_deref().unwrap_or("")
    }

    fn error_type(&self) -> &str {
        self.error_type.as_deref().unwrap_or("")
    }
}

/// Response for GET on base URL
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KiteTestResponse {
    pub status: String,
    pub data: String,
}

impl KiteResponseCheck for KiteTestResponse {
    fn status(&self) -> &str {
        &self.status
    }

    fn message(&self) -> &str {
        ""
    }

    fn error_type(&self) -> &str {
        ""
    }
}

/// Metadata for user login response
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KiteMeta {
    pub demat_consent: Option<String>,
}

/// User data returned in login response
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KiteLoginDataResult {
    pub user_type: String,
    pub email: String,
    pub user_name: String,
    pub user_shortname: String,
    pub broker: String,
    pub exchanges: Vec<String>,
    pub products: Vec<String>,
    pub order_types: Vec<String>,
    pub avatar_url: Option<String>,
    pub user_id: String,
    pub api_key: String,
    pub access_token: String,
    pub public_token: String,
    pub enctoken: String,
    pub refresh_token: Option<String>,
    pub silo: Option<String>,
    pub login_time: String,
    pub meta: KiteMeta,
}

/// Response for user login
pub type KiteLoginResponse = KiteResponse<KiteLoginDataResult>;

/// Convenience alias for responses that return a simple list.
pub type KiteListResponse<T> = KiteResponse<KiteList<T>>;
