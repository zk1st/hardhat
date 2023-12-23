use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use super::{
    custom_error::CustomError, source_location::SourceLocation, AnonymousContractFunction,
    PublicContractFunction,
};

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
    pub custom_errors: Vec<CustomError>,
    pub constructor: Option<Arc<AnonymousContractFunction>>,
    pub fallback: Option<Arc<AnonymousContractFunction>>,
    pub receive: Option<Arc<AnonymousContractFunction>>,
    pub selector_to_function: HashMap<[u8; 4], Arc<PublicContractFunction>>,
}
