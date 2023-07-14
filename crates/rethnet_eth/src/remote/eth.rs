#![cfg(feature = "serde")]

// Parts of this code were adapted from github.com/gakonst/ethers-rs and are distributed under its
// licenses:
// - https://github.com/gakonst/ethers-rs/blob/7e6c3ba98363bdf6131e8284f186cc2c70ff48c3/LICENSE-APACHE
// - https://github.com/gakonst/ethers-rs/blob/7e6c3ba98363bdf6131e8284f186cc2c70ff48c3/LICENSE-MIT
// For the original context, see https://github.com/gakonst/ethers-rs/tree/7e6c3ba98363bdf6131e8284f186cc2c70ff48c3

pub mod eip712;

use std::fmt::Debug;

use ethereum_types::U64;

use crate::{Address, Bloom, Bytes, B256, U256};

use super::{serde_with_helpers::optional_u64_from_hex, withdrawal::Withdrawal};

#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AccessListEntry {
    address: Address,
    storage_keys: Vec<U256>,
}

fn u64_from_hex<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    Ok(u64::from_str_radix(&s[2..], 16).expect("failed to parse u64"))
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub address: Address,
    pub topics: Vec<B256>,
    pub data: Bytes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<B256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<B256>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_u64_from_hex"
    )]
    pub transaction_index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_index: Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_log_index: Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    pub block_hash: Option<B256>,
    pub block_number: Option<U256>,
    pub contract_address: Option<Address>,
    pub cumulative_gas_used: U256,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_gas_price: Option<U256>,
    pub from: Address,
    pub gas_used: Option<U256>,
    pub logs: Vec<Log>,
    pub logs_bloom: Bloom,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<B256>,
    #[serde(deserialize_with = "optional_u64_from_hex")]
    pub status: Option<u64>,
    pub to: Option<Address>,
    pub transaction_hash: B256,
    #[serde(deserialize_with = "u64_from_hex")]
    pub transaction_index: u64,
    #[serde(
        rename = "type",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_u64_from_hex"
    )]
    pub transaction_type: Option<u64>,
}

/// Represents a JSON-RPC Ethereum block.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Block<TX> {
    /// Block hash
    pub hash: Option<B256>,
    /// Parent block hash
    pub parent_hash: B256,
    /// Ommers hash
    pub sha3_uncles: B256,
    /// Coinbase
    pub miner: Address,
    /// State root
    pub state_root: B256,
    /// Transactions root
    pub transactions_root: B256,
    /// Receipts root
    pub receipts_root: B256,
    /// Bloom filter
    pub logs_bloom: Bloom,
    /// Block difficulty
    pub difficulty: U256,
    /// Block number
    pub number: U256,
    /// Gas limit
    pub gas_limit: U256,
    /// Gas used
    pub gas_used: U256,
    /// Timestamp
    pub timestamp: U256,
    /// Extra data
    pub extra_data: Bytes,
    /// Mix hash
    pub mix_hash: B256,
    /// Nonce
    pub nonce: Option<U64>,
    /// Total difficulty
    pub total_difficulty: Option<U256>,
    /// Base fee per gas
    pub base_fee_per_gas: Option<U256>,
    /// Withdrawals root
    pub withdrawals_root: Option<B256>,
    /// Size
    pub size: U256,
    /// Transactions (can be hashes or full)
    #[serde(default = "Vec::new")]
    pub transactions: Vec<TX>,
    /// Withdrawals
    #[serde(default = "Vec::new")]
    pub withdrawals: Vec<Withdrawal>,
    /// Uncle hashes
    #[serde(default = "Vec::new")]
    pub uncles: Vec<B256>,
}
