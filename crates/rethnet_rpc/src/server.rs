use alloy_primitives::{Address, U256};
use rethnet_eth::remote::jsonrpc;

use crate::{handler::ToRpcError, EthRequest, EthRequestHandler};

fn to_rpc_response_data<T: serde::Serialize, E: ToRpcError>(
    result: Result<T, E>,
) -> jsonrpc::ResponseData<serde_json::Value> {
    match result {
        Ok(value) => match serde_json::to_value(value) {
            Ok(result) => jsonrpc::ResponseData::Success { result },
            Err(error) => jsonrpc::ResponseData::Error { error: todo!() },
        },
        Err(error) => jsonrpc::ResponseData::Error {
            error: error.to_rpc_error(),
        },
    }
}

pub struct EthServer<H: EthRequestHandler> {
    handler: H,
}

impl<H: EthRequestHandler> EthServer<H> {
    /// Handles the request and return JSON-RPC response
    pub fn handle_request(
        &self,
        id: jsonrpc::Id,
        request: EthRequest,
    ) -> jsonrpc::Response<serde_json::Value> {
        let data = match request {
            EthRequest::Accounts() => todo!(),
            EthRequest::BlockNumber() => todo!(),
            EthRequest::Call(_, _) => todo!(),
            EthRequest::ChainId() => todo!(),
            EthRequest::Coinbase() => todo!(),
            EthRequest::EstimateGas(_, _) => todo!(),
            EthRequest::FeeHistory(_, _, _) => todo!(),
            EthRequest::GasPrice() => todo!(),
            EthRequest::GetBalance(address, block) => {
                to_rpc_response_data(self.handle_get_balance(address, block_number))
            }
            EthRequest::GetBlockByNumber(_, _) => todo!(),
            EthRequest::GetBlockByHash(_, _) => todo!(),
            EthRequest::GetBlockTransactionCountByHash(_) => todo!(),
            EthRequest::GetBlockTransactionCountByNumber(_) => todo!(),
            EthRequest::GetCode(_, _) => todo!(),
            EthRequest::GetFilterChanges(_) => todo!(),
            EthRequest::GetFilterLogs(_) => todo!(),
            EthRequest::GetLogs(_) => todo!(),
            EthRequest::GetStorageAt(_, _, _) => todo!(),
            EthRequest::GetTransactionByBlockHashAndIndex(_, _) => todo!(),
            EthRequest::GetTransactionByBlockNumberAndIndex(_, _) => todo!(),
            EthRequest::GetTransactionByHash(_) => todo!(),
            EthRequest::GetTransactionCount(_, _) => todo!(),
            EthRequest::GetTransactionReceipt(_) => todo!(),
            EthRequest::Mining() => todo!(),
            EthRequest::NetVersion() => todo!(),
            EthRequest::NewBlockFilter() => todo!(),
            EthRequest::NewFilter(_) => todo!(),
            EthRequest::NewPendingTransactionFilter() => todo!(),
            EthRequest::PendingTransactions() => todo!(),
            EthRequest::SendRawTransaction(_) => todo!(),
            EthRequest::SendTransaction(_) => todo!(),
            EthRequest::Sign(_, _) => todo!(),
            EthRequest::SignTypedDataV4(_, _) => todo!(),
            EthRequest::Subscribe(_) => todo!(),
            EthRequest::Syncing() => todo!(),
            EthRequest::UninstallFilter(_) => todo!(),
            EthRequest::Unsubscribe(_) => todo!(),
        };

        jsonrpc::Response {
            jsonrpc: jsonrpc::Version::V2_0,
            id,
            data,
        }
    }

    pub fn handle_get_balance(
        &self,
        address: Address,
        block_number: BlockSpec,
    ) -> Result<U256, H::Error> {
        // Handle validation

        self.handler.balance(address, block_number)
    }
}
