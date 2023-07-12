#![cfg(feature = "serde")]

// Parts of this code were adapted from github.com/gakonst/ethers-rs and are distributed under its
// licenses:
// - https://github.com/gakonst/ethers-rs/blob/7e6c3ba98363bdf6131e8284f186cc2c70ff48c3/LICENSE-APACHE
// - https://github.com/gakonst/ethers-rs/blob/7e6c3ba98363bdf6131e8284f186cc2c70ff48c3/LICENSE-MIT
// For the original context, see https://github.com/gakonst/ethers-rs/tree/7e6c3ba98363bdf6131e8284f186cc2c70ff48c3

pub mod eip712;

use std::fmt::Debug;

use revm_primitives::ruint::aliases::B64;

use crate::{utils::B64Def, Address, Bloom, Bytes, B256, U256};

use super::{serde_with_helpers::optional_u64_from_hex, withdrawal::Withdrawal};

#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AccessListEntry {
    address: Address,
    storage_keys: Vec<U256>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// The transaction's hash
    pub hash: B256,
    pub nonce: U256,
    pub block_hash: Option<B256>,
    pub block_number: Option<U256>,
    #[serde(deserialize_with = "optional_u64_from_hex")]
    pub transaction_index: Option<u64>,
    pub from: Address,
    pub to: Option<Address>,
    pub value: U256,
    pub gas_price: Option<U256>,
    pub gas: U256,
    pub input: Bytes,
    #[serde(deserialize_with = "u64_from_hex")]
    pub v: u64,
    pub r: U256,
    pub s: U256,
    #[serde(default, deserialize_with = "optional_u64_from_hex")]
    pub chain_id: Option<u64>,
    #[serde(
        rename = "type",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_u64_from_hex"
    )]
    pub transaction_type: Option<u64>,
    #[serde(default)]
    pub access_list: Option<Vec<AccessListEntry>>,
    #[serde(default)]
    pub max_fee_per_gas: Option<U256>,
    #[serde(default)]
    pub max_priority_fee_per_gas: Option<U256>,
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

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Block<TX>
where
    TX: Debug + Default + Clone + PartialEq + Eq,
{
    pub hash: B256,
    pub parent_hash: B256,
    pub sha3_uncles: B256,
    pub miner: Address,
    pub state_root: B256,
    pub transactions_root: B256,
    pub receipts_root: B256,
    pub logs_bloom: Bloom,
    pub difficulty: U256,
    pub number: U256,
    pub gas_limit: U256,
    pub gas_used: U256,
    pub timestamp: U256,
    pub extra_data: Bytes,
    pub mix_hash: B256,
    #[serde(with = "B64Def")]
    pub nonce: B64,
    pub total_difficulty: Option<U256>,
    pub base_fee_per_gas: Option<U256>,
    pub withdrawals_root: Option<B256>,
    pub size: U256,
    #[serde(default)]
    pub transactions: Vec<TX>,
    #[serde(default)]
    pub withdrawals: Vec<Withdrawal>,
    #[serde(default)]
    pub uncles: Vec<B256>,
}
