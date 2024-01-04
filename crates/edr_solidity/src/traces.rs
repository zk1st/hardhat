use std::sync::Arc;

use edr_eth::U256;
use revm::primitives::{Bytes, ExecutionResult};

use crate::model::Contract;

struct EVMTraceOpcodeStep {
    pc: usize,
    instruction: u8
}

enum EVMTraceStep {
    EVMStep(EVMTraceOpcodeStep),
    CallStep(EVMTrace)
}

pub struct EVMTrace {
    address: Bytes,
    calldata: Bytes,
    value: U256,
    execution_result: ExecutionResult,
    steps: Vec<EVMTraceStep>,
}

enum SolidityTraceStep {
    InternalFunction,
    Modifier,
    Call(SolidityTrace),
}

enum ReturnData {
    Error(String),
    Panic(u64),
    Unknown(Bytes),
}

pub struct SolidityTrace {
    address: Bytes,
    contract: Option<Arc<Contract>>,
    steps: Vec<SolidityTraceStep>,
    return_data: ReturnData,
}

pub struct SolidityStackTrace {
    reason: InferredRevertReason,
    cause: Option<Box<SolidityStackTrace>>,
}

enum InferredRevertReason {
    RevertWithReason(String),
    RevertWithPanic(u64),
    RevertWithCustomError(Bytes),
    NonContractAccountCalled,
    Unknown,
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, fs};

    use edr_eth::Address;
    use revm::primitives::Output;
    use revm::primitives::ruint::aliases::B160;
    use versions::SemVer;

    use crate::solc::{
        model_builder::build_model,
        standard_json::{CompilerInput, CompilerOutput},
    };

    use super::*;

    enum TestTransactionTo {
        Address(Address),
        // send to the address of the contract
        // deployed in the given step
        Contract(u8),
    }

    struct TestTransaction {
        to: Option<TestTransactionTo>,
        data: Bytes,
    }

    struct TestStep {
        transaction: TestTransaction,
        expected: Option<SolidityStackTrace>,
    }

    struct Test {
        solc_version: SemVer,
        solc_input: String,
        solc_output: String,
        steps: Vec<TestStep>,
    }

    fn run_transaction(data: &Bytes, to: Option<Address>) -> (EVMTrace, Vec<String>) {
        // TODO run tx in edr instance
        unimplemented!()

        // TODO return evm trace result and console.logs
    }

    // return evm trace and console logs
    fn run_test_step(step: &TestStep, current_steps_results: &Vec<Option<Address>>) -> (EVMTrace, Vec<String>) {
        let to_address: Option<Address> = match step.transaction.to {
            Some(TestTransactionTo::Address(address)) => Some(address),
            Some(TestTransactionTo::Contract(step_index)) => {
                if let Some(address) = current_steps_results.get(step_index as usize).expect("Invalid step index") {
                    Some(address.clone())
                } else {
                    panic!("Contract address not found in previous steps results")
                }
            }
            None => None,
        };

        run_transaction(&step.transaction.data, to_address)
    }

    fn create_edr_instance() {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let unusedCacheDir = crate_root.join("test/cache-dir");

        // copy test config creation function from edr_provider/src/test_utils.rs

        let config = edr_provider::ProviderConfig::try_from(config)?;
        let runtime = runtime::Handle::current();

    }

    // Temporary function to run a stack trace test and get
    // the stack trace and the console logs.
    fn run_stack_trace_test(test: Test) {

        let edr_instance = create_edr_instance();

        // TODO create edr instance
        let codebase_model = build_model(test.solc_version, &test.solc_input, &test.solc_output);
        let mut steps_results: Vec<Option<Address>> = Vec::new();

        for step in test.steps {
            let (evm_trace, console_logs) = run_test_step(&step, &steps_results);

            if let ExecutionResult::Success { reason, gas_used, gas_refunded, logs, output } = evm_trace.execution_result {
                if let Output::Create(_, Some(deployed_contract_address)) = output {
                    let address = Address::from(deployed_contract_address.0);
                    steps_results.push(Some(address));
                } else {
                    steps_results.push(None);
                }
            } else {
                steps_results.push(None);
            }
        }
    }

    #[test]
    fn test_deploy_contract_and_call_function() {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let solc_input_path = crate_root.join("test/console-log-uint-solc-input.json");
        let solc_output_path = crate_root.join("test/console-log-uint-solc-output.json");

        let solc_input = fs::read_to_string(solc_input_path).expect("Unable to read solc input file");
        let solc_output = fs::read_to_string(solc_output_path).expect("Unable to read solc output file");

        let mut steps = Vec::new();

        /*
        Deploy this contract:

        // SPDX-License-Identifier: MIT
        pragma solidity ^0.8.0;

        import "hardhat/console.sol";

        contract Foo {
            function log() public {
                console.log(1);
            }
        }
         */
        steps.push(TestStep {
            transaction: TestTransaction {
                to: None,
                data: hex::decode("608060405234801561001057600080fd5b506101c8806100206000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c806351973ec914610030575b600080fd5b61003861003a565b005b6100446001610046565b565b6100dc8160405160240161005a9190610148565b6040516020818303038152906040527ff82c50f1000000000000000000000000000000000000000000000000000000007bffffffffffffffffffffffffffffffffffffffffffffffffffffffff19166020820180517bffffffffffffffffffffffffffffffffffffffffffffffffffffffff83818316178352505050506100df565b50565b6100f6816100ee6100f961011a565b63ffffffff16565b50565b60006a636f6e736f6c652e6c6f679050600080835160208501845afa505050565b610125819050919050565b61012d610163565b565b6000819050919050565b6101428161012f565b82525050565b600060208201905061015d6000830184610139565b92915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052605160045260246000fdfea2646970667358221220c2f50f27291df3c9ea5fe3f65e75f0fb8552d669d70b00101de42a5863df6bd464736f6c63430008110033").unwrap().into(),
            },
            expected: None,
        });

        // call log() in the contract deployed on the previous step
        steps.push(TestStep {
            transaction: TestTransaction {
                to: Some(TestTransactionTo::Contract(0)),
                data: hex::decode("51973ec9").unwrap().into()
            },
            expected: None,
        });

        run_stack_trace_test(Test {
            solc_version: SemVer::new("0.8.0").unwrap(),
            solc_input,
            solc_output,
            steps,
        })
    }
}
