use std::collections::HashMap;

use super::{
    bytecode::Bytecode, contract::Contract, custom_error::CustomError, function::Function,
    source_file::SourceFile, source_location::SourceLocation,
};

#[derive(Default, Debug)]
pub struct Codebase {
    pub source_files: HashMap<u32, SourceFile>,

    pub contracts: HashMap<u32, Contract>,
    pub functions: HashMap<u32, Function>,

    pub custom_errors: HashMap<u32, CustomError>,
    // TODO: compiler version
    pub bytecodes: Vec<Bytecode>,
}

impl Codebase {
    pub fn get_contract(source_location: &SourceLocation) -> Option<&Contract> {
        None
    }

    pub fn get_function(source_location: &SourceLocation) -> Option<&Function> {
        None
    }
}
