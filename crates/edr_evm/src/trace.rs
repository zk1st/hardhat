use std::{fmt::Debug, ops::Range};

use edr_eth::{Address, Bytes, U256};
use revm::{
    interpreter::{
        opcode, return_revert, CallInputs, CreateInputs, InstructionResult, Interpreter,
        InterpreterResult, SuccessOrHalt,
    },
    primitives::{Bytecode, ExecutionResult, Output},
    EvmContext, Inspector,
};

/// Stack tracing message
#[derive(Debug)]
pub enum TraceMessage {
    /// Event that occurs before a call or create message.
    Before(BeforeMessage),
    /// Event that occurs every step of a call or create message.
    Step(Step),
    /// Event that occurs after a call or create message.
    After(ExecutionResult),
}

/// Temporary before message type for handling traces
#[derive(Debug, Clone)]
pub struct BeforeMessage {
    /// Call depth
    pub depth: usize,
    /// Caller
    pub caller: Address,
    /// Callee
    pub to: Option<Address>,
    /// Transaction gas limit
    pub gas_limit: u64,
    /// Input data
    pub data: Bytes,
    /// Value
    pub value: U256,
    /// Code address
    pub code_address: Option<Address>,
    /// Bytecode
    pub code: Option<Bytecode>,
}

/// A trace for an EVM call.
#[derive(Debug, Default)]
pub struct Trace {
    // /// The individual steps of the call
    // pub steps: Vec<Step>,
    /// Messages
    pub messages: Vec<TraceMessage>,
    /// The return value of the call
    pub return_value: Bytes,
}

/// A single EVM step.
#[derive(Debug)]
pub struct Step {
    /// The program counter
    pub pc: u64,
    /// The call depth
    pub depth: u64,
    /// The executed op code
    pub opcode: u8,
    /// The top entry on the stack. None if the stack is empty.
    pub stack_top: Option<U256>,
    // /// The amount of gas that was used by the step
    // pub gas_cost: u64,
    // /// The amount of gas that was refunded by the step
    // pub gas_refunded: i64,
    // /// The contract being executed
    // pub contract: AccountInfo,
    // /// The address of the contract
    // pub contract_address: Address,
}

impl Trace {
    /// Adds a before message
    pub fn add_before(&mut self, message: BeforeMessage) {
        self.messages.push(TraceMessage::Before(message));
    }

    /// Adds a result message
    pub fn add_after(&mut self, result: ExecutionResult) {
        self.messages.push(TraceMessage::After(result));
    }

    /// Adds a VM step to the trace.
    pub fn add_step(&mut self, depth: u64, pc: usize, opcode: u8, stack_top: Option<U256>) {
        self.messages.push(TraceMessage::Step(Step {
            pc: pc as u64,
            depth,
            opcode,
            stack_top,
        }));
    }
}

/// Object that gathers trace information during EVM execution and can be turned
/// into a trace upon completion.
#[derive(Debug, Default)]
pub struct TraceCollector {
    trace: Trace,
    pending_before: Option<BeforeMessage>,
}

impl TraceCollector {
    /// Converts the [`TraceCollector`] into its [`Trace`].
    pub fn into_trace(self) -> Trace {
        self.trace
    }

    fn validate_before_message(&mut self) {
        if let Some(message) = self.pending_before.take() {
            self.trace.add_before(message);
        }
    }
}

impl<DatabaseError> Inspector<DatabaseError> for TraceCollector
where
    DatabaseError: Debug,
{
    fn call(
        &mut self,
        context: &mut EvmContext<'_, DatabaseError>,
        inputs: &mut CallInputs,
    ) -> Option<(InterpreterResult, Range<usize>)> {
        self.validate_before_message();

        // This needs to be split into two functions to avoid borrow checker issues
        #[allow(clippy::map_unwrap_or)]
        let code = context
            .journaled_state
            .state
            .get(&inputs.contract)
            .map(|account| account.info.clone())
            .map(|mut account_info| {
                if let Some(code) = account_info.code.take() {
                    code
                } else {
                    context.db.code_by_hash(account_info.code_hash).unwrap()
                }
            })
            .unwrap_or_else(|| {
                context.db.basic(inputs.contract).unwrap().map_or(
                    // If an invalid contract address was provided, return empty code
                    Bytecode::new(),
                    |account_info| {
                        account_info.code.unwrap_or_else(|| {
                            context.db.code_by_hash(account_info.code_hash).unwrap()
                        })
                    },
                )
            });

        self.pending_before = Some(BeforeMessage {
            depth: context.journaled_state.depth,
            caller: inputs.context.caller,
            to: Some(inputs.context.address),
            gas_limit: inputs.gas_limit,
            data: inputs.input.clone(),
            value: inputs.context.apparent_value,
            code_address: Some(inputs.context.code_address),
            code: Some(code),
        });

        None
    }

    fn call_end(
        &mut self,
        context: &mut EvmContext<'_, DatabaseError>,
        result: InterpreterResult,
    ) -> InterpreterResult {
        match result.result {
            return_revert!() if self.pending_before.is_some() => {
                self.pending_before = None;
                return result;
            }
            _ => (),
        }

        self.validate_before_message();

        let safe_ret = if result.result == InstructionResult::CallTooDeep
            || result.result == InstructionResult::OutOfFund
            || result.result == InstructionResult::StateChangeDuringStaticCall
        {
            InstructionResult::Revert
        } else {
            result.result
        };

        let execution_result = match safe_ret.into() {
            SuccessOrHalt::Success(reason) => ExecutionResult::Success {
                reason,
                gas_used: result.gas.spend(),
                gas_refunded: result.gas.refunded() as u64,
                logs: context.journaled_state.logs.clone(),
                output: Output::Call(result.output.clone()),
            },
            SuccessOrHalt::Revert => ExecutionResult::Revert {
                gas_used: result.gas.spend(),
                output: result.output.clone(),
            },
            SuccessOrHalt::Halt(reason) => ExecutionResult::Halt {
                reason,
                gas_used: result.gas.limit(),
            },
            SuccessOrHalt::InternalCallOrCreate | SuccessOrHalt::InternalContinue => {
                panic!("Internal error: {safe_ret:?}")
            }
            SuccessOrHalt::FatalExternalError => panic!("Fatal external error"),
        };

        self.trace.add_after(execution_result);

        result
    }

    fn create(
        &mut self,
        context: &mut EvmContext<'_, DatabaseError>,
        inputs: &mut CreateInputs,
    ) -> Option<(InterpreterResult, Option<Address>)> {
        self.validate_before_message();

        self.pending_before = Some(BeforeMessage {
            depth: context.journaled_state.depth,
            caller: inputs.caller,
            to: None,
            gas_limit: inputs.gas_limit,
            data: inputs.init_code.clone(),
            value: inputs.value,
            code_address: None,
            code: None,
        });

        None
    }

    fn create_end(
        &mut self,
        context: &mut EvmContext<'_, DatabaseError>,
        result: InterpreterResult,
        address: Option<Address>,
    ) -> (InterpreterResult, Option<Address>) {
        self.validate_before_message();

        let safe_ret = if result.result == InstructionResult::CallTooDeep
            || result.result == InstructionResult::OutOfFund
        {
            InstructionResult::Revert
        } else {
            result.result
        };

        let execution_result = match safe_ret.into() {
            SuccessOrHalt::Success(reason) => ExecutionResult::Success {
                reason,
                gas_used: result.gas.spend(),
                gas_refunded: result.gas.refunded() as u64,
                logs: context.journaled_state.logs.clone(),
                output: Output::Create(result.output.clone(), address),
            },
            SuccessOrHalt::Revert => ExecutionResult::Revert {
                gas_used: result.gas.spend(),
                output: result.output.clone(),
            },
            SuccessOrHalt::Halt(reason) => ExecutionResult::Halt {
                reason,
                gas_used: result.gas.limit(),
            },
            SuccessOrHalt::InternalCallOrCreate | SuccessOrHalt::InternalContinue => {
                panic!("Internal error: {safe_ret:?}")
            }
            SuccessOrHalt::FatalExternalError => panic!("Fatal external error"),
        };

        self.trace.add_after(execution_result);

        (result, address)
    }

    fn step(&mut self, interp: &mut Interpreter, context: &mut EvmContext<'_, DatabaseError>) {
        // Skip the step
        let skip_step = self.pending_before.as_ref().map_or(false, |message| {
            message.code.is_some() && interp.current_opcode() == opcode::STOP
        });

        self.validate_before_message();

        if !skip_step {
            self.trace.add_step(
                context.journaled_state.depth(),
                interp.program_counter(),
                interp.current_opcode(),
                interp.stack.data().last().cloned(),
            );
        }
    }
}
