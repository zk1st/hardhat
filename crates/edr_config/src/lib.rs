mod account;
mod blockchain;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    time::SystemTime,
};

use revm_primitives::{Address, SpecId, U256};

pub use k256;

pub use self::{
    account::AccountConfig,
    blockchain::{BlockchainConfig, ForkConfig},
};

/// Configuration of an EDR node.
pub struct NodeConfig {
    pub address: SocketAddr,
    pub accounts: Vec<AccountConfig>,
    pub allow_blocks_with_same_timestamp: bool,
    pub allow_unlimited_contract_size: bool,
    pub block_gas_limit: u64,
    pub blockchain: BlockchainConfig,
    pub cache_dir: PathBuf,
    pub chain_id: u64,
    pub coinbase: Address,
    pub gas: u64,
    pub hardfork: SpecId,
    pub initial_base_fee_per_gas: Option<U256>,
    pub initial_date: Option<SystemTime>,
    pub network_id: u64,
}
