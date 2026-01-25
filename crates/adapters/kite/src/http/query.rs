use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KiteTestParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct KiteTokenParams {
    pub api_key: String,
    pub request_token: String,
    pub checksum: String,
}
