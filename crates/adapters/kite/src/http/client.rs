use std::{collections::HashMap, num::NonZeroU32, sync::LazyLock};

use nautilus_core::consts::NAUTILUS_USER_AGENT;
use nautilus_network::{
    http::{HttpClient, Method, USER_AGENT},
    ratelimiter::quota::Quota,
    retry::{RetryConfig, RetryManager},
};
use serde::{Serialize, de::DeserializeOwned};
use tokio_util::sync::CancellationToken;

use crate::{
    common::credential::Credential,
    http::{
        error::KiteHttpError,
        models::{KiteLoginResponse, KiteResponseCheck},
        query::KiteTokenParams,
    },
};

pub static KITE_REST_QUOTA: LazyLock<Quota> = LazyLock::new(|| {
    Quota::per_second(NonZeroU32::new(1).expect("Should be a valid non-zero u32"))
});

pub static KITE_REPAY_QUOTA: LazyLock<Quota> = LazyLock::new(|| {
    Quota::per_second(NonZeroU32::new(1).expect("Should be a valid non-zero u32"))
});

/// Raw HTTP client for low-level Kite API operations.
#[allow(dead_code)]
#[derive(Clone)]
pub struct KiteRawHttpClient {
    base_url: String,
    client: HttpClient,
    /// (api_key, api_secret, access_token)
    credential: Option<Credential>,
    retry_manager: RetryManager<KiteHttpError>,
    cancellation_token: CancellationToken,
}

impl KiteRawHttpClient {
    /// Cancel all pending HTTP requests.
    pub fn cancel_all_requests(&self) {
        self.cancellation_token.cancel();
    }

    /// Get the cancellation token for this client.
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }

    /// Creates a new [`KiteRawHttpClient`] using the default Kite HTTP URL.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        base_url: Option<String>,
        timeout_secs: Option<u64>,
        max_retries: Option<u32>,
        retry_delay_ms: Option<u64>,
        retry_delay_max_ms: Option<u64>,
    ) -> Result<Self, KiteHttpError> {
        let retry_config = RetryConfig {
            max_retries: max_retries.unwrap_or(3),
            initial_delay_ms: retry_delay_ms.unwrap_or(1000),
            max_delay_ms: retry_delay_max_ms.unwrap_or(10_000),
            backoff_factor: 2.0,
            jitter_ms: 1000,
            operation_timeout_ms: Some(60_000),
            immediate_first: false,
            max_elapsed_ms: Some(180_000),
        };

        let retry_manager = RetryManager::new(retry_config);

        Ok(Self {
            base_url: base_url.unwrap_or_else(|| "https://api.kite.trade".to_string()),
            client: HttpClient::new(
                Self::default_headers(),
                vec![],
                Self::rate_limiter_quotas(),
                None,
                timeout_secs,
                None,
            )
            .map_err(|e| {
                KiteHttpError::NetworkError(format!("Failed to create HTTP client: {}", e))
            })?,
            credential: None,
            retry_manager,
            cancellation_token: CancellationToken::new(),
        })
    }

    /// Creates a new [`KiteRawHttpClient`] configured with credentials.
    #[allow(clippy::too_many_arguments)]
    pub fn with_credentials(
        base_url: Option<String>,
        api_key: String,
        api_secret: String,
        access_token: Option<String>,
        timeout_secs: Option<u64>,
        max_retries: Option<u32>,
        retry_delay_ms: Option<u64>,
        retry_delay_max_ms: Option<u64>,
    ) -> Result<Self, KiteHttpError> {
        let retry_config = RetryConfig {
            max_retries: max_retries.unwrap_or(3),
            initial_delay_ms: retry_delay_ms.unwrap_or(1000),
            max_delay_ms: retry_delay_max_ms.unwrap_or(10_000),
            backoff_factor: 2.0,
            jitter_ms: 1000,
            operation_timeout_ms: Some(60_000),
            immediate_first: false,
            max_elapsed_ms: Some(180_000),
        };

        let retry_manager = RetryManager::new(retry_config);

        Ok(Self {
            base_url: base_url.unwrap_or_else(|| "https://api.kite.trade".to_string()),
            client: HttpClient::new(
                Self::default_headers(),
                vec![],
                Self::rate_limiter_quotas(),
                None,
                timeout_secs,
                None,
            )
            .map_err(|e| {
                KiteHttpError::NetworkError(format!("Failed to create HTTP client: {}", e))
            })?,
            credential: Some(Credential::new(api_key, api_secret, access_token)),
            retry_manager,
            cancellation_token: CancellationToken::new(),
        })
    }

    fn default_headers() -> HashMap<String, String> {
        HashMap::from([(USER_AGENT.to_string(), NAUTILUS_USER_AGENT.to_string())])
    }

    fn rate_limiter_quotas() -> Vec<(String, Quota)> {
        vec![
            ("rest".to_string(), *KITE_REST_QUOTA),
            ("repay".to_string(), *KITE_REPAY_QUOTA),
        ]
    }

    #[allow(dead_code)]
    fn sign_request(&self) -> Result<HashMap<String, String>, KiteHttpError> {
        let credentials = self
            .credential
            .as_ref()
            .ok_or(KiteHttpError::MissingCredentials)?;

        let mut headers = HashMap::new();
        headers.insert("X-Kite-Version".to_string(), "3".to_string());
        headers.insert(
            "Authorization".to_string(),
            credentials.generate_auth_headers().to_string(),
        );

        Ok(headers)
    }

    #[allow(dead_code)]
    async fn send_request<T: DeserializeOwned + KiteResponseCheck, P: Serialize>(
        &self,
        method: Method,
        endpoint: &str,
        params: Option<&P>,
        body: Option<Vec<u8>>,
        authenticate: bool,
    ) -> Result<T, KiteHttpError> {
        let endpoint = endpoint.to_string();
        let url = format!("{}{endpoint}", self.base_url);
        let method_clone = method.clone();
        let body_clone = body.clone();

        let params_str = if method == Method::GET {
            params
                .map(serde_urlencoded::to_string)
                .transpose()
                .map_err(|e| KiteHttpError::JsonError(format!("Failed to serialize params: {e}")))?
        } else {
            None
        };

        let operation = || {
            let url = url.clone();
            let method = method_clone.clone();
            let body = body_clone.clone();
            let params_str = params_str.clone();

            async move {
                let mut headers = Self::default_headers();

                if authenticate {
                    let auth_headers = self.sign_request()?;
                    headers.extend(auth_headers);
                }

                if method == Method::POST || method == Method::PUT {
                    headers.insert(
                        "Content-Type".to_string(),
                        "application/x-www-form-urlencoded".to_string(),
                    );
                }

                let full_url = if let Some(ref query) = params_str {
                    if query.is_empty() {
                        url
                    } else {
                        format!("{url}?{query}")
                    }
                } else {
                    url
                };

                let response = self
                    .client
                    .request(method, full_url, None, Some(headers), body, None, None)
                    .await?;

                if response.status.as_u16() >= 400 {
                    let body = String::from_utf8_lossy(&response.body).to_string();
                    return Err(KiteHttpError::UnexpectedStatus {
                        status: response.status.as_u16(),
                        body,
                    });
                }

                println!("{:?}", response.body.clone());

                // Try to deserialize into the target type
                match serde_json::from_slice::<T>(&response.body) {
                    Ok(result) => {
                        // Check for API-level errors
                        if result.status() != "success" {
                            return Err(KiteHttpError::KiteError {
                                error_type: result.error_type().into(),
                                message: result.message().into(),
                            });
                        }

                        Ok(result)
                    }
                    Err(json_err) => Err(json_err.into()),
                }
            }
        };

        let should_retry = |error: &KiteHttpError| -> bool {
            match error {
                KiteHttpError::NetworkError(_) => true,
                KiteHttpError::UnexpectedStatus { status, .. } => *status >= 500,
                _ => false,
            }
        };

        let create_error = |msg: String| -> KiteHttpError {
            if msg == "canceled" {
                KiteHttpError::Canceled("Adapter disconnecting or shutting down".to_string())
            } else {
                KiteHttpError::NetworkError(msg)
            }
        };

        self.retry_manager
            .execute_with_retry_with_cancel(
                endpoint.as_str(),
                operation,
                should_retry,
                create_error,
                &self.cancellation_token,
            )
            .await
    }

    /// POST to /session/token to retrieve access token
    pub async fn get_access_token(&self) -> Result<KiteLoginResponse, KiteHttpError> {
        let request_token = "<your-request-token>";
        let checksum = self
            .credential
            .clone()
            .unwrap()
            .generate_checksum(request_token.to_string());
        let params = KiteTokenParams {
            api_key: self.credential.clone().unwrap().api_key().to_string(),
            request_token: request_token.to_string(),
            checksum: checksum.to_string(),
        };

        let body = serde_urlencoded::to_string(&params).unwrap();
        println!("{:?}", body.clone());
        self.send_request(
            Method::POST,
            "/session/token",
            Some(&params),
            Some(body.into_bytes()),
            false,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use crate::http::{models::KiteTestResponse, query::KiteTestParam};

    use super::*;

    // Test a http get to kite base url
    #[tokio::test]
    async fn test_get() {
        let client = KiteRawHttpClient::new(
            Some(String::from("https://api.kite.trade")),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let response: Result<KiteTestResponse, KiteHttpError> = client
            .send_request(
                Method::GET,
                "",
                Some(&KiteTestParam { parameter: None }),
                None,
                false,
            )
            .await;

        println!("{}", response.clone().unwrap().data);

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_get_access_token() {
        let client = KiteRawHttpClient::with_credentials(
            Some(String::from("https://api.kite.trade")),
            String::from("<your-api-key>"),
            String::from("<your-api-secret>"),
            None,
            None,
            None,
            None,
            None,
        );

        let response: Result<KiteLoginResponse, KiteHttpError> =
            client.unwrap().get_access_token().await;

        println!("{}", response.clone().unwrap().data.unwrap().access_token);

        assert!(response.is_ok());
    }
}
