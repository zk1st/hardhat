use std::{fmt::Debug, sync::Arc};

use rethnet_eth::{block::Header, Address, U256};
use revm::{
    db::DatabaseComponentError,
    primitives::{BlockEnv, CfgEnv, EVMError, ExecutionResult, InvalidTransaction, TxEnv},
    Inspector,
};
use tokio::runtime::Runtime;

use crate::{
    blockchain::AsyncBlockchain, evm::run_transaction, runtime::AsyncDatabase, state::AsyncState,
    trace::Trace, HeaderData,
};

#[derive(Debug, thiserror::Error)]
pub enum BlockTransactionError<BE, SE> {
    #[error(transparent)]
    BlockHash(BE),
    #[error("Transaction has a higher gas limit than the remaining gas in the block")]
    ExceedsBlockGasLimit,
    #[error("Invalid transaction")]
    InvalidTransaction(InvalidTransaction),
    #[error(transparent)]
    State(SE),
}

impl<BE, SE> From<EVMError<DatabaseComponentError<SE, BE>>> for BlockTransactionError<BE, SE>
where
    BE: Debug + Send + 'static,
    SE: Debug + Send + 'static,
{
    fn from(error: EVMError<DatabaseComponentError<SE, BE>>) -> Self {
        match error {
            EVMError::Transaction(e) => Self::InvalidTransaction(e),
            EVMError::PrevrandaoNotSet => unreachable!(),
            EVMError::Database(DatabaseComponentError::State(e)) => Self::State(e),
            EVMError::Database(DatabaseComponentError::BlockHash(e)) => Self::BlockHash(e),
        }
    }
}

/// A builder for constructing Ethereum blocks.
pub struct BlockBuilder<BE, SE>
where
    BE: Debug + Send + 'static,
    SE: Debug + Send + 'static,
{
    blockchain: Arc<AsyncBlockchain<BE>>,
    state: Arc<AsyncState<SE>>,
    cfg: CfgEnv,
    block: BlockEnv,
}

impl<BE, SE> BlockBuilder<BE, SE>
where
    BE: Debug + Send + 'static,
    SE: Debug + Send + 'static,
{
    /// Creates an intance of [`BlockBuilder`], creating a checkpoint in the process.
    pub fn new(
        blockchain: Arc<AsyncBlockchain<BE>>,
        state: Arc<AsyncState<SE>>,
        cfg: CfgEnv,
        parent: Header,
        header: HeaderData,
    ) -> Self {
        let block = BlockEnv {
            number: header.number.unwrap_or(parent.number + U256::from(1)),
            gas_limit: header.gas_limit.unwrap_or(parent.gas_limit),
            ..BlockEnv::default()
        };

        Self {
            blockchain,
            state,
            cfg,
            block,
        }
    }

    /// Retrieves the runtime of the [`BlockBuilder`].
    pub fn runtime(&self) -> &Runtime {
        self.state.runtime()
    }

    /// Adds a pending transaction to
    pub async fn add_transaction(
        &mut self,
        transaction: TxEnv,
        inspector: Option<Box<dyn Inspector<AsyncDatabase<BE, SE>> + Send>>,
    ) -> Result<(ExecutionResult, Trace), BlockTransactionError<BE, SE>> {
        let (result, changes, trace) = run_transaction(
            self.state.runtime(),
            self.blockchain.clone(),
            self.state.clone(),
            self.cfg.clone(),
            transaction,
            self.block.clone(),
            inspector,
        )
        .await
        .unwrap()?;

        self.state.apply(changes).await;

        Ok((result, trace))
    }

    /// Finalizes the block, paying rewards.
    pub async fn finalize(self, rewards: Vec<(Address, U256)>) -> Result<(), SE> {
        for (address, reward) in rewards {
            self.state
                .modify_account(
                    address,
                    Box::new(move |balance, _nonce, _code| *balance += reward),
                )
                .await?;
        }

        Ok(())
    }
}
