use std::{collections::HashMap, sync::Arc};

use super::{
    bytecode::Bytecode, contract::Contract, custom_error::CustomError, function::Function,
    ContractFunction, SourceFile, SourceLocation,
};

#[derive(Default, Debug)]
pub struct Codebase {
    pub functions: HashMap<u32, Arc<Function>>,
    pub contracts: HashMap<u32, Arc<Contract>>,
    pub source_files: HashMap<u32, SourceFile>,
    pub bytecodes: HashMap<u32, Bytecode>,
    // TODO: do we need to save the compiler version?
}

pub enum LookupResult {
    FreeFunction(Arc<Function>),
    Contract(Arc<Contract>),
    ContractFunction(Arc<Contract>, Arc<ContractFunction>),
}

impl Codebase {
    fn lookup(&self, location: &SourceLocation) -> Option<Arc<Function>> {
        let source_file = self.source_files.get(&location.file)?;

        let function = source_file.functions.get(location)?;

        Some(function.clone())
    }
}
