use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use versions::SemVer;

use super::{
    bytecode::Bytecode, contract::Contract, function::Function, ContractFunction, SourceFile,
    SourceLocation,
};

#[derive(Default, Debug)]
pub struct Codebase {
    pub functions: HashMap<u32, Arc<Function>>,
    pub contracts: HashMap<u32, Arc<Contract>>,
    pub source_files: HashMap<u32, SourceFile>,
    pub bytecodes: HashSet<Bytecode>,
    pub solc_version: SemVer,
}

#[allow(dead_code)]
pub enum LookupResult {
    FreeFunction(Arc<Function>),
    Contract(Arc<Contract>),
    ContractFunction(Arc<Contract>, Arc<ContractFunction>),
}

impl Codebase {
    #[allow(dead_code)]
    fn lookup(&self, location: &SourceLocation) -> Option<Arc<Function>> {
        let source_file = self.source_files.get(&location.file?)?;

        let function = source_file.functions.get(location)?;

        Some(function.clone())
    }
}
