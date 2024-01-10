// Based on Reth
// https://github.com/paradigmxyz/reth/blob/6ad221fd282f32d4bf699c92f22d4e2478372b5a/crates/revm/revm-inspectors/src/tracing/types.rs
// https://github.com/paradigmxyz/reth/blob/6ad221fd282f32d4bf699c92f22d4e2478372b5a/LICENSE-MIT

//! Types for representing call trace items.

use revm::interpreter::{CallContext, CallScheme, CreateScheme, InstructionResult, OpCode};
use serde::{Deserialize, Serialize};

use crate::{
    alloy_primitives::Log, tracing_inspector::utils::convert_memory, Address, Bytes, U256,
};

/// A trace of a call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallTrace {
    /// The depth of the call
    pub depth: usize,
    /// Whether the call was successful
    pub success: bool,
    /// caller of this call
    pub caller: Address,
    /// The destination address of the call or the address from the created
    /// contract.
    ///
    /// In other words, this is the callee if the [CallKind::Call] or the
    /// address of the created contract if [CallKind::Create].
    pub address: Address,
    /// Whether this is a call to a precompile
    ///
    /// Note: This is an Option because not all tracers make use of this
    pub maybe_precompile: Option<bool>,
    /// Holds the target for the selfdestruct refund target if `status` is
    /// [InstructionResult::SelfDestruct]
    pub selfdestruct_refund_target: Option<Address>,
    /// The kind of call this is
    pub kind: CallKind,
    /// The value transferred in the call
    pub value: U256,
    /// The calldata for the call, or the init code for contract creations
    pub data: Bytes,
    /// The return data of the call if this was not a contract creation,
    /// otherwise it is the runtime bytecode of the created contract
    pub output: Bytes,
    /// The gas cost of the call
    pub gas_used: u64,
    /// The gas limit of the call
    pub gas_limit: u64,
    /// The status of the trace's call
    pub status: InstructionResult,
    /// call context of the runtime
    pub call_context: Option<Box<CallContext>>,
    /// Opcode-level execution steps
    pub steps: Vec<CallTraceStep>,
}

impl Default for CallTrace {
    fn default() -> Self {
        Self {
            depth: Default::default(),
            success: Default::default(),
            caller: Default::default(),
            address: Default::default(),
            selfdestruct_refund_target: None,
            kind: Default::default(),
            value: Default::default(),
            data: Default::default(),
            maybe_precompile: None,
            output: Default::default(),
            gas_used: Default::default(),
            gas_limit: Default::default(),
            status: InstructionResult::Continue,
            call_context: Default::default(),
            steps: Default::default(),
        }
    }
}

/// A node in the collection
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct CallTraceNode {
    /// Parent node index in the collection
    pub parent: Option<usize>,
    /// Children node indexes in the collection
    pub children: Vec<usize>,
    /// This node's index in the collection
    pub idx: usize,
    /// The call trace
    pub trace: CallTrace,
    /// Recorded logs, if enabled
    pub logs: Vec<Log>,
    /// Ordering of child calls and logs
    pub ordering: Vec<LogCallOrder>,
}

/// A unified representation of a call.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CallKind {
    /// Represents a regular call.
    #[default]
    Call,
    /// Represents a static call.
    StaticCall,
    /// Represents a call code operation.
    CallCode,
    /// Represents a delegate call.
    DelegateCall,
    /// Represents a contract creation operation.
    Create,
    /// Represents a contract creation operation using the CREATE2 opcode.
    Create2,
}

impl CallKind {
    /// Returns true if the call is a create
    #[inline]
    pub fn is_any_create(&self) -> bool {
        matches!(self, CallKind::Create | CallKind::Create2)
    }

    /// Returns true if the call is a delegate of some sorts
    #[inline]
    pub fn is_delegate(&self) -> bool {
        matches!(self, CallKind::DelegateCall | CallKind::CallCode)
    }

    /// Returns true if the call is [CallKind::StaticCall].
    #[inline]
    pub fn is_static_call(&self) -> bool {
        matches!(self, CallKind::StaticCall)
    }
}

impl std::fmt::Display for CallKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallKind::Call => {
                write!(f, "CALL")
            }
            CallKind::StaticCall => {
                write!(f, "STATICCALL")
            }
            CallKind::CallCode => {
                write!(f, "CALLCODE")
            }
            CallKind::DelegateCall => {
                write!(f, "DELEGATECALL")
            }
            CallKind::Create => {
                write!(f, "CREATE")
            }
            CallKind::Create2 => {
                write!(f, "CREATE2")
            }
        }
    }
}

impl From<CallScheme> for CallKind {
    fn from(scheme: CallScheme) -> Self {
        match scheme {
            CallScheme::Call => CallKind::Call,
            CallScheme::StaticCall => CallKind::StaticCall,
            CallScheme::CallCode => CallKind::CallCode,
            CallScheme::DelegateCall => CallKind::DelegateCall,
        }
    }
}

impl From<CreateScheme> for CallKind {
    fn from(create: CreateScheme) -> Self {
        match create {
            CreateScheme::Create => CallKind::Create,
            CreateScheme::Create2 { .. } => CallKind::Create2,
        }
    }
}

/// Ordering enum for calls and logs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogCallOrder {
    /// Contains the index of the corresponding log
    Log(usize),
    /// Contains the index of the corresponding trace node
    Call(usize),
}

/// Represents a tracked call step during execution
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallTraceStep {
    // Fields filled in `step`
    /// Call depth
    pub depth: u64,
    /// Program counter before step execution
    pub pc: usize,
    /// Opcode to be executed
    pub op: OpCode,
    /// Current contract address
    pub contract: Address,
    /// Stack before step execution
    pub stack: Option<Vec<U256>>,
    /// The new stack items placed by this step if any
    pub push_stack: Option<Vec<U256>>,
    /// All allocated memory in a step
    ///
    /// This will be empty if memory capture is disabled
    pub memory: RecordedMemory,
    /// Size of memory at the beginning of the step
    pub memory_size: usize,
    /// Remaining gas before step execution
    pub gas_remaining: u64,
    /// Gas refund counter before step execution
    pub gas_refund_counter: u64,
    // Fields filled in `step_end`
    /// Gas cost of step execution
    pub gas_cost: u64,
    /// Change of the contract state after step execution (effect of the
    /// SLOAD/SSTORE instructions)
    pub storage_change: Option<StorageChange>,
    /// Final status of the step
    ///
    /// This is set after the step was executed.
    pub status: InstructionResult,
}

/// Represents the source of a storage change - e.g., whether it came
/// from an SSTORE or SLOAD instruction.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageChangeReason {
    /// SLOAD opcode
    SLOAD,
    /// SSTORE opcode
    SSTORE,
}

/// Represents a storage change during execution.
///
/// This maps to evm internals:
/// [JournalEntry::StorageChange](revm::JournalEntry::StorageChange)
///
/// It is used to track both storage change and warm load of a storage slot. For
/// warm load in regard to EIP-2929 AccessList had_value will be None.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StorageChange {
    /// key of the storage slot
    pub key: U256,
    /// Current value of the storage slot
    pub value: U256,
    /// The previous value of the storage slot, if any
    pub had_value: Option<U256>,
    /// How this storage was accessed
    pub reason: StorageChangeReason,
}

/// Represents the memory captured during execution
///
/// This is a wrapper around the [SharedMemory](revm::interpreter::SharedMemory)
/// context memory.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecordedMemory(pub(crate) Vec<u8>);

impl RecordedMemory {
    // #[inline]
    // pub(crate) fn new(mem: Vec<u8>) -> Self {
    //     Self(mem)
    // }

    /// Returns the memory as a byte slice
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    // #[inline]
    // pub(crate) fn resize(&mut self, size: usize) {
    //     self.0.resize(size, 0);
    // }

    /// Returns the size of the memory
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the memory is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts the memory into 32byte hex chunks
    #[inline]
    pub fn memory_chunks(&self) -> Vec<String> {
        convert_memory(self.as_bytes())
    }
}

impl AsRef<[u8]> for RecordedMemory {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
