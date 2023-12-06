use std::collections::HashMap;

use super::{
    contract::Contract, function::Function, source_file::SourceFile, source_map::SourceMap,
};

pub struct Codebase {
    // A mapping from source file IDs (as provided by solc's output) to source file.
    source_files: HashMap<u32, SourceFile>,

    // A vector of contracts, where the vector index is used as contract ID.
    contracts: Vec<Contract>,
    // functions?
    // custom errors?

    // compiler version
}

impl Codebase {
    pub fn get_contract(source_map: &SourceMap) -> Option<&Contract> {
        None
    }

    pub fn get_function(source_map: &SourceMap) -> Option<&Function> {
        None
    }
}
