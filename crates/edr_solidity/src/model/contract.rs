use super::source_map::SourceMap;

pub enum ContractType {
    Library,
    Contract,
}

pub struct Contract {
    pub name: String,
    pub contract_type: ContractType,
    pub source_map: SourceMap,
    // local function ids
    // constructor, fallback, receive
    // custom errors
    // selector to function id

    // add linearized base contract functions
    // add linearized base contract custom errors?
}
