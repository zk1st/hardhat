use std::fmt::Debug;

use revm::{
    db::{DatabaseComponents, StateRef},
    primitives::{BlockEnv, CfgEnv, ExecutionResult, ResultAndState, SpecId, TxEnv},
    EvmBuilder,
};

use crate::{
    blockchain::SyncBlockchain,
    inspector::SyncInspector,
    state::{StateOverrides, StateRefOverrider, SyncState},
    transaction::TransactionError,
};

pub trait DebuggableStateRef: StateRef + Debug {}

impl<T: StateRef + Debug> DebuggableStateRef for T {}

/// Asynchronous implementation of the Database super-trait
pub type SyncDatabase<'blockchain, 'state, BlockchainErrorT, StateErrorT> = DatabaseComponents<
    &'state dyn DebuggableStateRef<Error = StateErrorT>,
    &'blockchain dyn SyncBlockchain<BlockchainErrorT, StateErrorT>,
>;

/// Runs a transaction without committing the state.
#[cfg_attr(feature = "tracing", tracing::instrument(skip(inspector)))]
pub fn dry_run<BlockchainErrorT, StateErrorT>(
    blockchain: &dyn SyncBlockchain<BlockchainErrorT, StateErrorT>,
    state: &dyn SyncState<StateErrorT>,
    state_overrides: &StateOverrides,
    spec_id: SpecId,
    cfg: CfgEnv,
    transaction: TxEnv,
    block: BlockEnv,
    inspector: Option<&mut dyn SyncInspector<BlockchainErrorT, StateErrorT>>,
) -> Result<ResultAndState, TransactionError<BlockchainErrorT, StateErrorT>>
where
    BlockchainErrorT: Debug + Send,
    StateErrorT: Debug + Send,
{
    if spec_id > SpecId::MERGE && block.prevrandao.is_none() {
        return Err(TransactionError::MissingPrevrandao);
    }

    if transaction.gas_priority_fee.is_some() && spec_id < SpecId::LONDON {
        return Err(TransactionError::Eip1559Unsupported);
    }

    let state_overrider = StateRefOverrider::new(state_overrides, &state);

    let mut evm = EvmBuilder::default()
        .with_ref_db(DatabaseComponents {
            state,
            block_hash: blockchain,
        })
        .with_spec_id(spec_id)
        .modify_block_env(|evm_block| {
            *evm_block = block;
        })
        .modify_cfg_env(|evm_cfg| {
            *evm_cfg = cfg;
        })
        .modify_tx_env(|evm_tx| {
            *evm_tx = transaction;
        })
        .build();

    evm.transact().map_err(TransactionError::from)
}

/// Runs a transaction without committing the state, while disabling balance
/// checks and creating accounts for new addresses.
#[cfg_attr(feature = "tracing", tracing::instrument(skip(inspector)))]
pub fn guaranteed_dry_run<BlockchainErrorT, StateErrorT>(
    blockchain: &dyn SyncBlockchain<BlockchainErrorT, StateErrorT>,
    state: &dyn SyncState<StateErrorT>,
    state_overrides: &StateOverrides,
    spec_id: SpecId,
    mut cfg: CfgEnv,
    transaction: TxEnv,
    block: BlockEnv,
    inspector: Option<&mut dyn SyncInspector<BlockchainErrorT, StateErrorT>>,
) -> Result<ResultAndState, TransactionError<BlockchainErrorT, StateErrorT>>
where
    BlockchainErrorT: Debug + Send,
    StateErrorT: Debug + Send,
{
    cfg.disable_balance_check = true;
    cfg.disable_block_gas_limit = true;
    dry_run(
        blockchain,
        state,
        state_overrides,
        spec_id,
        cfg,
        transaction,
        block,
        inspector,
    )
}

/// Runs a transaction, committing the state in the process.
#[cfg_attr(feature = "tracing", tracing::instrument(skip(inspector)))]
pub fn run<BlockchainErrorT, StateErrorT>(
    blockchain: &dyn SyncBlockchain<BlockchainErrorT, StateErrorT>,
    state: &mut dyn SyncState<StateErrorT>,
    spec_id: SpecId,
    cfg: CfgEnv,
    transaction: TxEnv,
    block: BlockEnv,
    inspector: Option<&mut dyn SyncInspector<BlockchainErrorT, StateErrorT>>,
) -> Result<ExecutionResult, TransactionError<BlockchainErrorT, StateErrorT>>
where
    BlockchainErrorT: Debug + Send,
    StateErrorT: Debug + Send,
{
    let ResultAndState {
        result,
        state: changes,
    } = dry_run(
        blockchain,
        state,
        &StateOverrides::default(),
        spec_id,
        cfg,
        transaction,
        block,
        inspector,
    )?;

    state.commit(changes);

    Ok(result)
}
