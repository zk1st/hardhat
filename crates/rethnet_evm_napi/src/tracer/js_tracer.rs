use napi::bindgen_prelude::{BigInt, Buffer};
use napi_derive::napi;
use rethnet_evm::{
    trace::{BeforeMessage, Step},
    OPCODE_JUMPMAP,
};

use crate::{account::Account, transaction::result::ExecutionResult};

#[napi(object)]
pub struct TracingMessage {
    /// Recipient address. None if it is a Create message.
    #[napi(readonly)]
    pub to: Option<Buffer>,

    /// Depth of the message
    #[napi(readonly)]
    pub depth: u8,

    /// Input data of the message
    #[napi(readonly)]
    pub data: Buffer,

    /// Value sent in the message
    #[napi(readonly)]
    pub value: BigInt,

    /// Address of the code that is being executed. Can be different from `to` if a delegate call
    /// is being done.
    #[napi(readonly)]
    pub code_address: Option<Buffer>,

    /// Code of the contract that is being executed.
    #[napi(readonly)]
    pub code: Option<Buffer>,
}

#[napi(object)]
pub struct TracingStep {
    /// Call depth
    #[napi(readonly)]
    pub depth: u8,
    /// The program counter
    #[napi(readonly)]
    pub pc: BigInt,
    /// The executed op code
    #[napi(readonly)]
    pub opcode: String,
    // /// The return value of the step
    // #[napi(readonly)]
    // pub return_value: u8,
    /// The amount of gas that was used by the step
    #[napi(readonly)]
    pub gas_cost: BigInt,
    /// The amount of gas that was refunded by the step
    #[napi(readonly)]
    pub gas_refunded: BigInt,
    /// The amount of gas left
    #[napi(readonly)]
    pub gas_left: BigInt,
    /// The stack
    #[napi(readonly)]
    pub stack: Vec<BigInt>,
    /// The memory
    #[napi(readonly)]
    pub memory: Buffer,
    /// The contract being executed
    #[napi(readonly)]
    pub contract: Account,
    /// The address of the contract
    #[napi(readonly)]
    pub contract_address: Buffer,
    // /// The address of the code being executed
    // #[napi(readonly)]
    // pub code_address: Buffer,
}

#[napi(object)]
pub struct TracingMessageResult {
    /// Execution result
    #[napi(readonly)]
    pub execution_result: ExecutionResult,
}

impl From<BeforeMessage> for TracingMessage {
    fn from(message: BeforeMessage) -> Self {
        TracingMessage {
            to: message.to.map(|to| Buffer::from(to.to_vec())),
            depth: message.depth as u8,
            data: Buffer::from(Vec::from(message.data)),
            value: BigInt {
                sign_bit: false,
                words: message.value.into_limbs().to_vec(),
            },
            code_address: message
                .code_address
                .map(|address| Buffer::from(address.to_vec())),
            code: message
                .code
                .map(|code| Buffer::from(Vec::from(code.original_bytes()))),
        }
    }
}

impl From<Step> for TracingStep {
    fn from(step: Step) -> Self {
        Self {
            depth: step.depth as u8,
            pc: BigInt::from(step.pc),
            opcode: OPCODE_JUMPMAP[usize::from(step.opcode)]
                .unwrap_or("")
                .to_string(),
            gas_cost: BigInt::from(0u64),
            gas_refunded: BigInt::from(0u64),
            gas_left: BigInt::from(0u64),
            stack: Vec::new(),
            memory: Buffer::from(Vec::new()),
            contract: Account::from(step.contract),
            contract_address: Buffer::from(step.contract_address.to_vec()),
        }
    }
}
