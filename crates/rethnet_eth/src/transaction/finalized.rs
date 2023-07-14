use std::ops::Deref;

use revm_primitives::{Address, B256, U256};

use super::SignedTransaction;

/// Represents a transaction that's part of a finalized block.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FinalizedTransaction {
    /// The hash of the block where the given transaction was included.
    pub block_hash: B256,
    /// The number of the block where the given transaction was included.
    pub block_number: U256,
    /// Address of the sender.
    pub from: Address,
    /// Hash of the transaction.
    pub hash: B256,
    /// Signed transaction
    #[cfg_attr(feature = "serde", serde(flatten))]
    transaction: SignedTransaction,
    /// The index of the transaction within the block.
    pub transaction_index: usize,
}

impl Deref for FinalizedTransaction {
    type Target = SignedTransaction;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}
