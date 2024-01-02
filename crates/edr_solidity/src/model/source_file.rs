use std::{collections::HashMap, sync::Arc};

use super::{Contract, Function, SourceLocation};

#[derive(Debug)]
pub struct SourceFile {
    pub source_name: String,

    // Note: Content can be optimized away. See: https://github.com/NomicFoundation/hardhat/commit/53be7fef20773a7b9e5c4e6630f41c120b898b8f
    pub content: String,

    // Note: These hashmaps are here to have a similar representation of the owneship
    //  model that we need, but in reality we need something like an interval tree for each of
    // them.
    pub functions: HashMap<SourceLocation, Arc<Function>>,
    pub contracts: HashMap<SourceLocation, Arc<Contract>>,
}

impl SourceFile {
    pub fn new(source_name: String) -> Self {
        Self {
            source_name,
            // TODO do we need the content?
            content: "".to_string(),
            functions: HashMap::default(),
            contracts: HashMap::default(),
        }
    }
}
