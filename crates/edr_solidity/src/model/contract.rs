use std::collections::{HashMap, HashSet};

use super::{custom_error::CustomError, source_location::SourceLocation};

#[derive(Debug)]
pub enum ContractType {
    Library,
    Contract,
}

#[derive(Debug)]
pub struct Contract {
    pub name: String,
    pub contract_type: ContractType,
    pub location: SourceLocation,
    pub local_function_ids: HashSet<u32>,
    pub linearized_base_contract_ids: Vec<u32>,
    pub custom_errors: Vec<CustomError>, // TODO: Maybe use ids?

    // Linearized inheritance
    pub constructor_id: Option<u32>,
    pub fallback_id: Option<u32>,
    pub receive_id: Option<u32>,
    pub selector_to_function_id: HashMap<[u8; 4], u32>,
    // add linearized base contract custom errors?
}

impl Contract {
    pub fn new(
        name: String,
        contract_type: ContractType,
        location: SourceLocation,
        linearized_base_contract_ids: Vec<u32>,
    ) -> Contract {
        Contract {
            name,
            contract_type,
            location,
            local_function_ids: HashSet::default(),
            constructor_id: None,
            fallback_id: None,
            receive_id: None,
            custom_errors: Vec::default(),
            selector_to_function_id: HashMap::default(),
            linearized_base_contract_ids,
        }
    }
}
