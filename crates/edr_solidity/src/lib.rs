#![warn(missing_docs)]

//! Repository of information about contracts written in Solidity.

/// ABI definitions for Ethereum
pub mod abi;
/// Model of the project's codebase
pub mod build_model;
/// Map of bytecodes to known contracts
pub mod contracts_identifier;
mod opcodes;

// Re-export types from `alloy-sol-types`
pub use alloy_sol_types::{Error, SolInterface};
