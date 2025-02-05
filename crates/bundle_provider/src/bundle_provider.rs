use alloy::primitives::{Bytes, B256};
use jsonrpsee::http_client::HttpClient;
use jsonrpsee::{core::client::ClientT, rpc_params};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct BundleClient {
    client: HttpClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthSendBundleResponse {
    pub bundle_hash: B256,
}

impl BundleClient {
    pub fn new(url: impl AsRef<str>) -> Self {
        let client = HttpClient::builder()
            .build(url)
            .expect("failed to connect to RPC provider");
        Self { client }
    }

    pub async fn send_bundle(&self, bundle: EthSendBundle) -> Result<(), String> {
        // Result contents optional because some endpoints don't return this response
        let res: Result<Option<EthSendBundleResponse>, _> = self
            .client
            .request("eth_sendBundle", rpc_params![bundle])
            .await;
        if let Err(e) = res {
            return Err(format!("Failed to send bundle: {:?}", e));
        }

        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthSendBundle {
    /// A list of hex-encoded signed transactions
    pub txs: Vec<Bytes>,
    /// hex-encoded block number for which this bundle is valid
    #[serde(with = "alloy_serde::quantity")]
    pub block_number: u64,
    /// unix timestamp when this bundle becomes active
    #[serde(
        default,
        with = "alloy_serde::quantity::opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub min_timestamp: Option<u64>,
    /// unix timestamp how long this bundle stays valid
    #[serde(
        default,
        with = "alloy_serde::quantity::opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_timestamp: Option<u64>,
    /// list of hashes of possibly reverting txs
    #[serde(
        default
        // this doesn't work on rbuilder:
        // , skip_serializing_if = "Vec::is_empty"
    )]
    pub reverting_tx_hashes: Vec<B256>,
    /// UUID that can be used to cancel/replace this bundle
    #[serde(
        default,
        rename = "replacementUuid",
        skip_serializing_if = "Option::is_none"
    )]
    pub replacement_uuid: Option<String>,
}

impl EthSendBundle {
    pub fn new_basic(txs: Vec<Bytes>, block_number: u64) -> Self {
        Self {
            txs,
            block_number,
            min_timestamp: None,
            max_timestamp: None,
            reverting_tx_hashes: Vec::new(),
            replacement_uuid: None,
        }
    }

    pub async fn send_to_builder(&self, client: &BundleClient) -> Result<(), String> {
        client.send_bundle(self.clone()).await
    }
}
