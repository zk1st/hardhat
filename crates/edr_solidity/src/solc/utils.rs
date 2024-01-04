use std::sync::Arc;

use super::ast::{self, ContractKind};
use crate::model::{AnonymousContractFunction, ContractType, SourceLocation};

#[derive(Debug)]
pub(crate) struct ContractData {
    pub name: String,
    pub contract_type: ContractType,
    pub location: SourceLocation,
    pub linearized_base_contract_ids: Vec<u32>,
    pub local_function_ids: Vec<u32>,
    pub constructor: Option<Arc<AnonymousContractFunction>>,
    pub fallback: Option<Arc<AnonymousContractFunction>>,
    pub receive: Option<Arc<AnonymousContractFunction>>,
}

pub fn parse_hex_function_selector(selector: &str) -> Option<[u8; 4]> {
    if selector.len() == 8 {
        let mut bytes = [0u8; 4];
        hex::decode_to_slice(selector, &mut bytes).ok()?;

        Some(bytes)
    } else {
        None
    }
}

// Returns the contract type for a given contract kind, or None if we don't
// support the contract kind.
pub fn contract_kind_to_contract_type(contract_kind: &ContractKind) -> Option<ContractType> {
    match contract_kind {
        ContractKind::Contract => Some(ContractType::Contract),
        ContractKind::Interface => None,
        ContractKind::Library => Some(ContractType::Library),
    }
}

pub fn parse_ast_src_location(src_location: &ast::SourceLocation) -> SourceLocation {
    let values: Vec<&str> = src_location.split(':').collect();

    let invalid_location_msg =
        format!("Invalid src field found in the compilation output: {src_location}");

    assert!(values.len() >= 3, "{invalid_location_msg}");

    SourceLocation {
        file: Some(values[2].parse().expect(&invalid_location_msg)),
        offset: values[0].parse().expect(&invalid_location_msg),
        length: values[1].parse().expect(&invalid_location_msg),
    }
}
