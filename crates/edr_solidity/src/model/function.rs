use super::source_location::SourceLocation;

#[derive(Debug)]
pub enum Function {
    FreeFunction(FreeFunction),
    ContractFunction(ContractFunction),
}

#[derive(Debug)]
pub struct FreeFunction {
    pub name: String,
    pub location: SourceLocation,
}

#[derive(Debug)]
pub enum ContractFunction {
    Constructor(AnonymousContractFunction),
    Fallback(AnonymousContractFunction),
    Receive(AnonymousContractFunction),
    Modifier(InternalContractFunction),
    Getter(PublicContractFunction),
    PublicFunction(PublicContractFunction),
    InternalFunction(InternalContractFunction),
}

#[derive(Debug)]
pub struct AnonymousContractFunction {
    pub name: String,
    pub location: SourceLocation,
    pub contract_id: u32,
    pub is_public: bool,
}

#[derive(Debug)]
pub struct PublicContractFunction {
    pub name: String,
    pub location: SourceLocation,
    pub contract_id: u32,
    pub payable: bool,
    pub selector: [u8; 4],
    pub method_identifier: String,
}

#[derive(Debug)]
pub struct InternalContractFunction {
    pub name: String,
    pub location: SourceLocation,
    pub contract_id: u32,
}

impl PublicContractFunction {
    pub fn is_valid_calldata(calldata: &[u8]) -> bool {
        true
    }
}
