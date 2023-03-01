mod builder;

use rethnet_eth::{block::Block, receipt::TypedReceipt, U256};

use crate::transaction::TransactionInfo;

pub use builder::BlockBuilder;

/// Container type that gathers all block data
#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub block: Block,
    pub transactions: Vec<TransactionInfo>,
    pub receipts: Vec<TypedReceipt>,
}

/// Data of a block header
pub struct HeaderData {
    /// The block number
    pub number: Option<U256>,
    /// The block's gas limit
    pub gas_limit: Option<U256>,
}
