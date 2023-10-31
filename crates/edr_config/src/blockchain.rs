use std::collections::HashMap;

/// Configuration options for an EDR blockchain.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BlockchainConfig {
    pub forking: Option<ForkConfig>,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self { forking: None }
    }
}

/// Configuration options for a forked EDR node.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForkConfig {
    pub json_rpc_url: String,
    pub block_number: Option<u64>,
    pub http_headers: Option<HashMap<String, String>>,
}
