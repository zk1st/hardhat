#![allow(clippy::enum_variant_names)]

use serde::Deserialize;

pub type SourceLocation = String;

#[derive(Debug, Deserialize)]
pub struct SourceUnit {
    pub id: i32,
    pub src: SourceLocation,
    pub nodes: Vec<TopLevelNode>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "nodeType")]
#[serde(rename_all = "camelCase")]
pub enum TopLevelNode {
    ContractDefinition(ContractDefinition),
    FunctionDefinition(FunctionDefinition),
    VariableDeclaration(VariableDeclaration),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TopLevelNodeOrOther {
    Node(TopLevelNode),
    Other(serde_json::Value),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractDefinition {
    pub id: i32,
    pub src: SourceLocation,
    pub name: String,
    pub contract_kind: ContractKind,
    pub linearized_base_contracts: Vec<i32>,
    pub nodes: Vec<ContractNode>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ContractNodeOrOther {
    Node(TopLevelNode),
    Other(serde_json::Value),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "nodeType")]
#[serde(rename_all = "camelCase")]
pub enum ContractNode {
    FunctionDefinition(FunctionDefinition),
    ModifierDefinition(ModifierDefinition),
    VariableDeclaration(VariableDeclaration),
}

#[derive(Debug, Deserialize)]
pub enum ContractKind {
    Contract,
    Interface,
    Library,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDefinition {
    pub id: i32,
    pub src: SourceLocation,
    pub name: String,
    pub function_selector: Option<String>,
    pub implemented: bool,
    pub kind: FunctionKind,
    pub parameters: ParameterList,
    pub return_parameters: ParameterList,
    pub state_mutability: StateMutability,
    pub visibility: Visibility,
}

#[derive(Debug, Deserialize)]
pub enum FunctionKind {
    Function,
    Receive,
    Constructor,
    Fallback,
    FreeFunction,
}

#[derive(Debug, Deserialize)]
pub enum StateMutability {
    Payable,
    Pure,
    Nonpayable,
    View,
}

#[derive(Debug, Deserialize)]
pub struct ParameterList {
    pub id: i32,
    pub src: SourceLocation,
    pub parameters: Vec<VariableDeclaration>,
}

#[derive(Debug, Deserialize)]

pub enum Visibility {
    External,
    Public,
    Internal,
    Private,
}

#[derive(Debug, Deserialize)]
pub struct ModifierDefinition {
    pub id: i32,
    pub src: SourceLocation,
    pub name: String,
    pub parameters: ParameterList,
    pub visibility: Visibility,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableDeclaration {
    pub id: i32,
    pub src: SourceLocation,
    pub name: String,
    pub function_selector: Option<String>,
    pub indexed: Option<bool>,
    pub type_name: Option<TypeName>,
    pub visibility: Visibility,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "nodeType")]
#[serde(rename_all = "camelCase")]
pub enum TypeName {
    ArrayTypeName(ArrayTypeName),
    ElementaryTypeName(ElementaryTypeName),
    FunctionTypeName(FunctionTypeName),
    Mapping(Mapping),
    UserDefinedTypeName(UserDefinedTypeName),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrayTypeName {
    pub id: i32,
    pub src: SourceLocation,
    pub type_descriptions: TypeDescriptions,
    pub base_type: Box<TypeName>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementaryTypeName {
    pub id: i32,
    pub src: SourceLocation,
    pub type_descriptions: TypeDescriptions,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionTypeName {
    pub id: i32,
    pub src: SourceLocation,
    pub type_descriptions: TypeDescriptions,
    pub parameter_types: ParameterList,
    pub return_parameter_types: ParameterList,
    pub state_mutability: StateMutability,
    pub visibility: Visibility,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mapping {
    pub id: i32,
    pub src: SourceLocation,
    pub type_descriptions: TypeDescriptions,
    pub key_type: Box<TypeName>,
    pub value_type: Box<TypeName>,
    pub key_name: Option<String>,
    pub key_name_location: Option<String>,
    pub value_name: Option<String>,
    pub value_name_location: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDefinedTypeName {
    pub id: i32,
    pub src: SourceLocation,
    pub type_descriptions: TypeDescriptions,
    pub name: Option<String>,
    pub referenced_declaration: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDescriptions {
    pub type_identifier: Option<String>,
    pub type_string: Option<String>,
}
