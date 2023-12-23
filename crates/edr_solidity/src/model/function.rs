use std::sync::Arc;

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

// Note: Some of these have Arc because we share them between contracts
#[derive(Debug)]
pub enum ContractFunction {
    Constructor(Arc<AnonymousContractFunction>),
    Fallback(Arc<AnonymousContractFunction>),
    Receive(Arc<AnonymousContractFunction>),
    Modifier(InternalContractFunction),
    Getter(Arc<PublicContractFunction>),
    PublicFunction(Arc<PublicContractFunction>),
    InternalFunction(InternalContractFunction),
}

#[derive(Debug)]
pub struct AnonymousContractFunction {
    pub location: SourceLocation,
    pub public: bool,
}

#[derive(Debug)]
pub struct PublicContractFunction {
    pub name: String,
    pub location: SourceLocation,
    pub payable: bool,
    pub selector: [u8; 4],

    // Note: Having the method identifier is enough to represent its abi, that's why
    //   I'm saving it here, not because we need it, we don't. This should be replaced
    //   with the propoer internal representation of the ABI and the right public mehtods.
    pub method_identifier: String,
}

#[derive(Debug)]
pub struct InternalContractFunction {
    pub name: String,
    pub location: SourceLocation,
}
