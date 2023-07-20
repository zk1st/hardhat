use alloy_primitives::{Address, Bytes, ChainId, StorageValue, B256, U256};
use rethnet_eth::remote::jsonrpc;

/// Trait to convert errors to RPC errors.
pub trait ToRpcError {
    /// Converts the error to an RPC error
    fn to_rpc_error(self) -> jsonrpc::Error;
}

/// Trait for handling Ethereum requests.
pub trait EthRequestHandler {
    /// The handler's error type
    type Error: ToRpcError;

    /// Returns the current client version.
    ///
    /// Handler for ETH RPC call: `web3_clientVersion`
    fn client_version(&self) -> Result<String, Self::Error>;

    /// Returns Keccak-256 (not the standardized SHA3-256) of the given data.
    ///
    /// Handler for ETH RPC call: `web3_sha3`
    fn sha3(&self, bytes: Bytes) -> Result<String, Self::Error>;

    /// Returns the client coinbase address.
    ///
    /// Handler for ETH RPC call: `eth_coinbase`
    fn coinbase(&self) -> Result<Address, Self::Error>;

    /// Returns true if client is actively mining new blocks.
    ///
    /// Handler for ETH RPC call: `eth_mining`
    fn is_mining(&self) -> Result<bool, Self::Error>;

    /// Returns the chain ID used for transaction signing at the
    /// current best block. None is returned if not
    /// available.
    ///
    /// Handler for ETH RPC call: `eth_chainId`
    fn chain_id(&self) -> Result<Option<ChainId>, Self::Error>;

    /// Returns the current network ID.
    ///
    /// Handler for ETH RPC call: `eth_networkId`
    fn network_id(&self) -> Result<Option<ChainId>, Self::Error>;

    /// Returns true if client is actively listening for network connections.
    ///
    /// Handler for ETH RPC call: `net_listening`
    fn net_listening(&self) -> Result<bool, Self::Error>;

    /// Returns a fee per gas that is an estimate of how much you can pay as a priority fee, or
    /// 'tip', to get a transaction included in the current block.
    ///
    /// Handler for ETH RPC call: `eth_maxPriorityFeePerGas`
    fn max_priority_fee_per_gas(&self) -> Result<U256, Self::Error>;

    /// Returns the accounts list
    ///
    /// Handler for ETH RPC call: `eth_accounts`
    fn accounts(&self) -> Result<Vec<Address>, Self::Error>;

    /// Returns the number of most recent block.
    ///
    /// Handler for ETH RPC call: `eth_blockNumber`
    fn block_number(&self) -> Result<U256, Self::Error>;

    /// Returns balance of the given account.
    ///
    /// Handler for ETH RPC call: `eth_getBalance`
    fn balance(&self, address: Address, block_number: Option<BlockId>)
        -> Result<U256, Self::Error>;

    /// Returns content of the storage at given address.
    ///
    /// Handler for ETH RPC call: `eth_getStorageAt`
    fn storage_at(
        &self,
        address: Address,
        index: U256,
        block_number: Option<BlockId>,
    ) -> Result<StorageValue, Self::Error>;

    /// Returns block with given hash.
    ///
    /// Handler for ETH RPC call: `eth_getBlockByHash`
    fn block_by_hash(&self, hash: B256) -> Result<Option<Block<TxHash>>, Self::Error>;

    /// Returns a _full_ block with given hash.
    ///
    /// Handler for ETH RPC call: `eth_getBlockByHash`
    fn block_by_hash_full(&self, hash: B256) -> Result<Option<Block<Transaction>>, Self::Error>;

    /// Returns block with given number.
    ///
    /// Handler for ETH RPC call: `eth_getBlockByNumber`
    fn block_by_number(&self, number: BlockNumber) -> Result<Option<Block<TxHash>>, Self::Error>;

    /// Returns a _full_ block with given number
    ///
    /// Handler for ETH RPC call: `eth_getBlockByNumber`
    fn block_by_number_full(
        &self,
        number: BlockNumber,
    ) -> Result<Option<Block<Transaction>>, Self::Error>;

    /// Returns the number of transactions sent from given address at given time (block number).
    ///
    /// Also checks the pending transactions if `block_number` is
    /// `BlockId::Number(BlockNumber::Pending)`
    ///
    /// Handler for ETH RPC call: `eth_getTransactionCount`
    fn transaction_count(
        &self,
        address: Address,
        block_number: Option<BlockId>,
    ) -> Result<U256, Self::Error>;

    /// Returns the number of transactions in a block with given hash.
    ///
    /// Handler for ETH RPC call: `eth_getBlockTransactionCountByHash`
    fn block_transaction_count_by_hash(&self, hash: B256) -> Result<Option<U256>, Self::Error>;

    /// Returns the number of transactions in a block with given block number.
    ///
    /// Handler for ETH RPC call: `eth_getBlockTransactionCountByNumber`
    fn block_transaction_count_by_number(
        &self,
        block_number: BlockNumber,
    ) -> Result<Option<U256>, Self::Error>;

    /// Returns the number of uncles in a block with given hash.
    ///
    /// Handler for ETH RPC call: `eth_getUncleCountByBlockHash`
    fn block_uncles_count_by_hash(&self, hash: B256) -> Result<U256, Self::Error>;

    /// Returns the number of uncles in a block with given block number.
    ///
    /// Handler for ETH RPC call: `eth_getUncleCountByBlockNumber`
    fn block_uncles_count_by_number(&self, block_number: BlockNumber) -> Result<U256, Self::Error>;

    /// Returns the code at given address at given time (block number).
    ///
    /// Handler for ETH RPC call: `eth_getCode`
    fn get_code(
        &self,
        address: Address,
        block_number: Option<BlockId>,
    ) -> Result<Bytes, Self::Error>;

    /// Returns the account and storage values of the specified account including the Merkle-proof.
    /// This call can be used to verify that the data you are pulling from is not tampered with.
    ///
    /// Handler for ETH RPC call: `eth_getProof`
    fn get_proof(
        &self,
        address: Address,
        keys: Vec<B256>,
        block_number: Option<BlockId>,
    ) -> Result<AccountProof, Self::Error>;

    /// Signs data via [EIP-712](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-712.md).
    ///
    /// Handler for ETH RPC call: `eth_signTypedData`
    fn sign_typed_data(
        &self,
        _address: Address,
        _data: serde_json::Value,
    ) -> Result<String, Self::Error>;

    /// Signs data via [EIP-712](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-712.md).
    ///
    /// Handler for ETH RPC call: `eth_signTypedData_v3`
    fn sign_typed_data_v3(
        &self,
        _address: Address,
        _data: serde_json::Value,
    ) -> Result<String, Self::Error>;

    /// Signs data via [EIP-712](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-712.md), and includes full support of arrays and recursive data structures.
    ///
    /// Handler for ETH RPC call: `eth_signTypedData_v4`
    fn sign_typed_data_v4(&self, address: Address, data: &TypedData)
        -> Result<String, Self::Error>;

    /// The sign method calculates an Ethereum specific signature
    ///
    /// Handler for ETH RPC call: `eth_sign`
    fn sign(&self, address: Address, content: impl AsRef<[u8]>) -> Result<String, Self::Error>;

    /// Signs a transaction
    ///
    /// Handler for ETH RPC call: `eth_signTransaction`
    fn sign_transaction(&self, request: EthTransactionRequest) -> Result<String, Self::Error>;

    /// Sends a transaction
    ///
    /// Handler for ETH RPC call: `eth_sendTransaction`
    fn send_transaction(&self, request: EthTransactionRequest) -> Result<TxHash, Self::Error>;

    /// Sends signed transaction, returning its hash.
    ///
    /// Handler for ETH RPC call: `eth_sendRawTransaction`
    fn send_raw_transaction(&self, tx: Bytes) -> Result<TxHash, Self::Error>;

    /// Call contract, returning the output data.
    ///
    /// Handler for ETH RPC call: `eth_call`
    fn call(
        &self,
        request: EthTransactionRequest,
        block_number: Option<BlockId>,
        overrides: Option<StateOverride>,
    ) -> Result<Bytes, Self::Error>;

    /// This method creates an EIP2930 type accessList based on a given Transaction. The accessList
    /// contains all storage slots and addresses read and written by the transaction, except for the
    /// sender account and the precompiles.
    ///
    /// It returns list of addresses and storage keys used by the transaction, plus the gas
    /// consumed when the access list is added. That is, it gives you the list of addresses and
    /// storage keys that will be used by that transaction, plus the gas consumed if the access
    /// list is included. Like eth_estimateGas, this is an estimation; the list could change
    /// when the transaction is actually mined. Adding an accessList to your transaction does
    /// not necessary result in lower gas usage compared to a transaction without an access
    /// list.
    ///
    /// Handler for ETH RPC call: `eth_createAccessList`
    fn create_access_list(
        &self,
        mut request: EthTransactionRequest,
        block_number: Option<BlockId>,
    ) -> Result<AccessListWithGasUsed, Self::Error>;

    /// Estimate gas needed for execution of given contract.
    /// If no block parameter is given, it will use the pending block by default
    ///
    /// Handler for ETH RPC call: `eth_estimateGas`
    fn estimate_gas(
        &self,
        request: EthTransactionRequest,
        block_number: Option<BlockId>,
    ) -> Result<U256, Self::Error>;

    /// Get transaction by its hash.
    ///
    /// This will check the storage for a matching transaction, if no transaction exists in storage
    /// this will also scan the mempool for a matching pending transaction
    ///
    /// Handler for ETH RPC call: `eth_getTransactionByHash`
    fn transaction_by_hash(&self, hash: B256) -> Result<Option<Transaction>, Self::Error>;

    /// Returns transaction at given block hash and index.
    ///
    /// Handler for ETH RPC call: `eth_getTransactionByBlockHashAndIndex`
    fn transaction_by_block_hash_and_index(
        &self,
        hash: B256,
        index: Index,
    ) -> Result<Option<Transaction>, Self::Error>;

    /// Returns transaction by given block number and index.
    ///
    /// Handler for ETH RPC call: `eth_getTransactionByBlockNumberAndIndex`
    fn transaction_by_block_number_and_index(
        &self,
        block: BlockNumber,
        idx: Index,
    ) -> Result<Option<Transaction>, Self::Error>;

    /// Returns transaction receipt by transaction hash.
    ///
    /// Handler for ETH RPC call: `eth_getTransactionReceipt`
    fn transaction_receipt(&self, hash: B256) -> Result<Option<TransactionReceipt>, Self::Error>;

    /// Returns an uncles at given block and index.
    ///
    /// Handler for ETH RPC call: `eth_getUncleByBlockHashAndIndex`
    fn uncle_by_block_hash_and_index(
        &self,
        block_hash: B256,
        idx: Index,
    ) -> Result<Option<Block<TxHash>>, Self::Error>;

    /// Returns an uncles at given block and index.
    ///
    /// Handler for ETH RPC call: `eth_getUncleByBlockNumberAndIndex`
    fn uncle_by_block_number_and_index(
        &self,
        block_number: BlockNumber,
        idx: Index,
    ) -> Result<Option<Block<TxHash>>, Self::Error>;

    /// Returns logs matching given filter object.
    ///
    /// Handler for ETH RPC call: `eth_getLogs`
    fn logs(&self, filter: Filter) -> Result<Vec<Log>, Self::Error>;

    /// Returns the hash of the current block, the seedHash, and the boundary condition to be met.
    ///
    /// Handler for ETH RPC call: `eth_getWork`
    fn work(&self) -> Result<Work, Self::Error>;

    /// Returns the sync status, always be fails.
    ///
    /// Handler for ETH RPC call: `eth_syncing`
    fn syncing(&self) -> Result<bool, Self::Error>;

    /// Used for submitting a proof-of-work solution.
    ///
    /// Handler for ETH RPC call: `eth_submitWork`
    fn submit_work(&self, _: H64, _: B256, _: B256) -> Result<bool, Self::Error>;

    /// Used for submitting mining hashrate.
    ///
    /// Handler for ETH RPC call: `eth_submitHashrate`
    fn submit_hashrate(&self, _: U256, _: B256) -> Result<bool, Self::Error>;

    /// Introduced in EIP-1159 for getting information on the appropriate priority fee to use.
    ///
    /// Handler for ETH RPC call: `eth_feeHistory`
    fn fee_history(
        &self,
        block_count: U256,
        newest_block: BlockNumber,
        reward_percentiles: Vec<f64>,
    ) -> Result<FeeHistory, Self::Error>;

    /// Introduced in EIP-1159, a Geth-specific and simplified priority fee oracle.
    /// Leverages the already existing fee history cache.
    ///
    /// Returns a suggestion for a gas tip cap for dynamic fee transactions.
    ///
    /// Handler for ETH RPC call: `eth_maxPriorityFeePerGas`
    fn max_priority_fee_per_gas(&self) -> Result<U256, Self::Error>;

    /// Creates a filter object, based on filter options, to notify when the state changes (logs).
    ///
    /// Handler for ETH RPC call: `eth_newFilter`
    fn new_filter(&self, filter: Filter) -> Result<String, Self::Error>;

    /// Creates a filter in the node, to notify when a new block arrives.
    ///
    /// Handler for ETH RPC call: `eth_newBlockFilter`
    fn new_block_filter(&self) -> Result<String, Self::Error>;

    /// Creates a filter in the node, to notify when new pending transactions arrive.
    ///
    /// Handler for ETH RPC call: `eth_newPendingTransactionFilter`
    fn new_pending_transaction_filter(&self) -> Result<String, Self::Error>;

    /// Polling method for a filter, which returns an array of logs which occurred since last poll.
    ///
    /// Handler for ETH RPC call: `eth_getFilterChanges`
    fn get_filter_changes(&self, id: &str) -> ResponseResult;

    /// Returns an array of all logs matching filter with given id.
    ///
    /// Handler for ETH RPC call: `eth_getFilterLogs`
    fn get_filter_logs(&self, id: &str) -> Result<Vec<Log>, Self::Error>;

    /// Handler for ETH RPC call: `eth_uninstallFilter`
    fn uninstall_filter(&self, id: &str) -> Result<bool, Self::Error>;

    /// Returns traces for the transaction hash for geth's tracing endpoint
    ///
    /// Handler for RPC call: `debug_traceTransaction`
    fn debug_trace_transaction(
        &self,
        tx_hash: B256,
        opts: GethDebugTracingOptions,
    ) -> Result<GethTrace, Self::Error>;
}
