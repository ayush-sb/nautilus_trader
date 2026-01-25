use nautilus_network::http::HttpClientError;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

/// Build error for query parameter validation.
#[derive(Debug, Clone, Error)]
pub enum KiteBuildError {
    /// Missing required category.
    #[error("Missing required category")]
    MissingCategory,
    /// Missing required symbol.
    #[error("Missing required symbol")]
    MissingSymbol,
}

/// Represents the JSON structure of an error response returned by the Kite API.
/// # References
/// - <https://kite.trade/docs/connect/v3/exceptions/>
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KiteErrorResponse {
    /// A human-readable explanation of the error condition.
    pub message: String,
    /// Error type returned by Kite.
    pub error_type: KiteErrorType,
}

/// A typed error enumeration for the Kite HTTP client.
#[derive(Debug, Clone, Error)]
pub enum KiteHttpError {
    /// Error variant when credentials are missing but the request is authenticated.
    #[error("Missing credentials for authenticated request")]
    MissingCredentials,
    /// Errors returned directly by Kite (non-zero code).
    #[error("Kite error {error_type}: {message}")]
    KiteError {
        error_type: KiteErrorType,
        message: String,
    },
    /// Failure during JSON serialization/deserialization.
    #[error("JSON error: {0}")]
    JsonError(String),
    /// Parameter validation error.
    #[error("Parameter validation error: {0}")]
    ValidationError(String),
    /// Build error for query parameters.
    #[error("Build error: {0}")]
    BuildError(#[from] KiteBuildError),
    /// Request was canceled, typically due to shutdown or disconnect.
    #[error("Request canceled: {0}")]
    Canceled(String),
    /// Generic network error (for retries, cancellations, etc).
    #[error("Network error: {0}")]
    NetworkError(String),
    /// Any unknown HTTP status or unexpected response from Bybit.
    #[error("Unexpected HTTP status code {status}: {body}")]
    UnexpectedStatus { status: u16, body: String },
}

/// Types of HTTP errors returned directly by Kite.
///
/// # References
/// - <https://kite.trade/docs/connect/v3/exceptions/>
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum KiteErrorType {
    /// Preceded by a 403 header, this indicates the expiry or
    /// invalidation of an authenticated session.
    TokenException,
    /// Represents user account related errors.
    UserException,
    /// Represents order related errors such placement failures, a corrupt
    /// fetch, etc.
    OrderException,
    /// Represents missing required fields, bad values for parameters, etc.
    InputException,
    /// Represents insufficient funds, required for the order placement.
    MarginException,
    /// Represents insufficient holdings, available to place sell order for
    /// specified instrument.
    HoldingException,
    /// Represents a network error where the API was unable to communicate with
    /// the OMS (Order Management System).
    NetworkException,
    /// Represents an internal system error where the API was unable to
    /// understand the response from the OMS to inturn respond to a request.
    DataException,
    /// Represents an unclassified error. This should only happen rarely.
    GeneralException,
}

impl Display for KiteErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KiteErrorType::TokenException => write!(f, "TokenException"),
            KiteErrorType::UserException => write!(f, "UserException"),
            KiteErrorType::OrderException => write!(f, "OrderException"),
            KiteErrorType::InputException => write!(f, "InputException"),
            KiteErrorType::MarginException => write!(f, "MarginException"),
            KiteErrorType::HoldingException => write!(f, "HoldingException"),
            KiteErrorType::NetworkException => write!(f, "NetworkException"),
            KiteErrorType::DataException => write!(f, "DataException"),
            KiteErrorType::GeneralException => write!(f, "GeneralException"),
        }
    }
}

impl From<&str> for KiteErrorType {
    fn from(value: &str) -> Self {
        match value {
            "TokenException" => KiteErrorType::TokenException,
            "UserException" => KiteErrorType::UserException,
            "OrderException" => KiteErrorType::OrderException,
            "InputException" => KiteErrorType::InputException,
            "MarginException" => KiteErrorType::MarginException,
            "HoldingException" => KiteErrorType::HoldingException,
            "NetworkException" => KiteErrorType::NetworkException,
            "DataException" => KiteErrorType::DataException,
            "GeneralException" => KiteErrorType::GeneralException,
            _ => KiteErrorType::GeneralException,
        }
    }
}

impl From<HttpClientError> for KiteHttpError {
    fn from(error: HttpClientError) -> Self {
        Self::NetworkError(error.to_string())
    }
}

impl From<String> for KiteHttpError {
    fn from(error: String) -> Self {
        Self::ValidationError(error)
    }
}

impl From<serde_json::Error> for KiteHttpError {
    fn from(error: serde_json::Error) -> Self {
        Self::JsonError(error.to_string())
    }
}

impl From<KiteErrorResponse> for KiteHttpError {
    fn from(error: KiteErrorResponse) -> Self {
        Self::KiteError {
            error_type: error.error_type,
            message: error.message,
        }
    }
}
