use super::ast::{self, ContractKind};
use crate::model::{ContractType, SourceLocation};

pub fn parse_hex_function_selector(selector: &String) -> Option<[u8; 4]> {
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
    let values: Vec<&str> = src_location.split(":").collect();

    let invalid_location_msg =
        format!("Invalid src field found in the compilation output: {src_location}");

    assert!(values.len() >= 3, "{invalid_location_msg}");

    SourceLocation {
        file: values[2].parse().expect(&invalid_location_msg),
        offset: values[0].parse().expect(&invalid_location_msg),
        length: values[1].parse().expect(&invalid_location_msg),
    }
}
