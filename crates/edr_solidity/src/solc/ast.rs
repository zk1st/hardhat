use serde::{de::DeserializeOwned, Deserialize, Deserializer};

pub type SourceLocation = String;

#[derive(Debug, Deserialize)]
pub struct SourceUnit {
    pub id: u32,
    pub src: SourceLocation,
    #[serde(deserialize_with = "deserialize_top_level_nodes")]
    pub nodes: Vec<Option<TopLevelNode>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "nodeType")]
pub enum TopLevelNode {
    ContractDefinition(ContractDefinition),
    FunctionDefinition(FunctionDefinition),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractDefinition {
    pub id: u32,
    pub src: SourceLocation,
    pub name: String,
    pub contract_kind: ContractKind,
    pub linearized_base_contracts: Vec<u32>,
    #[serde(deserialize_with = "deserialize_contract_definition_nodes")]
    pub nodes: Vec<Option<ContractNode>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "nodeType")]
pub enum ContractNode {
    FunctionDefinition(FunctionDefinition),
    ModifierDefinition(ModifierDefinition),
    VariableDeclaration(VariableDeclaration),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContractKind {
    Contract,
    Interface,
    Library,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDefinition {
    pub id: u32,
    pub src: SourceLocation,
    pub name: String,
    pub function_selector: Option<String>,
    pub implemented: bool,
    pub kind: FunctionKind,
    pub parameters: ParameterList,
    pub state_mutability: StateMutability,
    pub visibility: Visibility,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FunctionKind {
    Function,
    Receive,
    Constructor,
    Fallback,
    FreeFunction,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum StateMutability {
    Payable,
    Pure,
    Nonpayable,
    View,
}

#[derive(Debug, Deserialize)]
pub struct ParameterList {
    pub id: u32,
    pub src: SourceLocation,
    pub parameters: Vec<VariableDeclaration>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]

pub enum Visibility {
    External,
    Public,
    Internal,
    Private,
}

#[derive(Debug, Deserialize)]
pub struct ModifierDefinition {
    pub id: u32,
    pub src: SourceLocation,
    pub name: String,
    pub parameters: ParameterList,
    pub visibility: Visibility,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableDeclaration {
    pub id: u32,
    pub src: SourceLocation,
    pub name: String,
    pub function_selector: Option<String>,
    pub indexed: Option<bool>,
    pub type_name: Option<TypeName>,
    pub visibility: Visibility,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "nodeType")]
pub enum TypeName {
    #[serde(rename = "ArrayTypeName")]
    Array(ArrayTypeName),
    #[serde(rename = "ElementaryTypeName")]
    Elementary(ElementaryTypeName),
    #[serde(rename = "FunctionTypeName")]
    Function(FunctionTypeName),
    Mapping(Mapping),
    #[serde(rename = "UserDefinedTypeName")]
    UserDefined(UserDefinedTypeName),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrayTypeName {
    pub id: u32,
    pub src: SourceLocation,
    pub type_descriptions: TypeDescriptions,
    pub base_type: Box<TypeName>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementaryTypeName {
    pub id: u32,
    pub src: SourceLocation,
    pub type_descriptions: TypeDescriptions,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionTypeName {
    pub id: u32,
    pub src: SourceLocation,
    pub type_descriptions: TypeDescriptions,
    pub parameter_types: ParameterList,
    pub state_mutability: StateMutability,
    pub visibility: Visibility,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mapping {
    pub id: u32,
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
    pub id: u32,
    pub src: SourceLocation,
    pub type_descriptions: TypeDescriptions,
    pub name: Option<String>,
    pub referenced_declaration: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDescriptions {
    pub type_identifier: Option<String>,
    pub type_string: Option<String>,
}

fn deserialize_top_level_nodes<'de, D>(
    deserializer: D,
) -> Result<Vec<Option<TopLevelNode>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_non_exhaustive_enum_vec(
        deserializer,
        "nodeType",
        &["ContractDefinition", "FunctionDefinition"],
    )
}

fn deserialize_contract_definition_nodes<'de, D>(
    deserializer: D,
) -> Result<Vec<Option<ContractNode>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_non_exhaustive_enum_vec(
        deserializer,
        "nodeType",
        &[
            "FunctionDefinition",
            "ModifierDefinition",
            "VariableDeclaration",
        ],
    )
}

pub fn deserialize_non_exhaustive_enum_vec<'de, D, T>(
    deserializer: D,
    tag_name: &'static str,
    variants: &'static [&'static str],
) -> Result<Vec<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    struct NonExhaustiveVisitor<T> {
        tag_name: &'static str,
        variants: &'static [&'static str],
        marker: std::marker::PhantomData<T>,
    }

    impl<'v, T> serde::de::Visitor<'v> for NonExhaustiveVisitor<T>
    where
        T: DeserializeOwned,
    {
        type Value = Vec<Option<T>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("A sequence of values that can be deserialized into a vec of enums where some variants are unknown")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'v>,
        {
            let mut values = Vec::new();

            while let Some(value) = seq.next_element::<serde_json::Value>()? {
                let is_known_variant = value
                    .get(self.tag_name)
                    .and_then(|value| value.as_str())
                    .map_or(false, |value| self.variants.contains(&value));

                if is_known_variant {
                    let value = serde_json::from_value::<T>(value).map_err(|err| {
                        serde::de::Error::custom(format!("Failed to deserialize value: {err:?}"))
                    })?;
                    values.push(Some(value));
                } else {
                    values.push(None);
                }
            }

            Ok(values)
        }
    }

    deserializer.deserialize_seq(NonExhaustiveVisitor {
        tag_name,
        variants,
        marker: std::marker::PhantomData,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_source_unit() {
        let json = r#"{
            "absolutePath": "c.sol",
            "exportedSymbols": {
              "C": [
                10
              ]
            },
            "id": 11,
            "nodeType": "SourceUnit",
            "nodes": [
              {
                "id": 1,
                "literals": [
                  "solidity",
                  "^",
                  "0.6",
                  ".0"
                ],
                "nodeType": "PragmaDirective",
                "src": "0:23:0"
              },
              {
                "abstract": false,
                "baseContracts": [],
                "contractDependencies": [],
                "contractKind": "contract",
                "documentation": null,
                "fullyImplemented": true,
                "id": 10,
                "linearizedBaseContracts": [
                  10
                ],
                "name": "C",
                "nodeType": "ContractDefinition",
                "nodes": [
                  {
                    "constant": false,
                    "functionSelector": "f0fdf834",
                    "id": 5,
                    "name": "a",
                    "nodeType": "VariableDeclaration",
                    "overrides": null,
                    "scope": 10,
                    "src": "42:30:0",
                    "stateVariable": true,
                    "storageLocation": "default",
                    "typeDescriptions": {
                      "typeIdentifier": "t_mapping$_t_uint256_$_t_uint256_$",
                      "typeString": "mapping(uint256 => uint256)"
                    },
                    "typeName": {
                      "id": 4,
                      "keyType": {
                        "id": 2,
                        "name": "uint",
                        "nodeType": "ElementaryTypeName",
                        "src": "50:4:0",
                        "typeDescriptions": {
                          "typeIdentifier": "t_uint256",
                          "typeString": "uint256"
                        }
                      },
                      "nodeType": "Mapping",
                      "src": "42:21:0",
                      "typeDescriptions": {
                        "typeIdentifier": "t_mapping$_t_uint256_$_t_uint256_$",
                        "typeString": "mapping(uint256 => uint256)"
                      },
                      "valueType": {
                        "id": 3,
                        "name": "uint",
                        "nodeType": "ElementaryTypeName",
                        "src": "58:4:0",
                        "typeDescriptions": {
                          "typeIdentifier": "t_uint256",
                          "typeString": "uint256"
                        }
                      }
                    },
                    "value": null,
                    "visibility": "public"
                  },
                  {
                    "body": {
                      "id": 8,
                      "nodeType": "Block",
                      "src": "98:5:0",
                      "statements": []
                    },
                    "documentation": null,
                    "id": 9,
                    "implemented": true,
                    "kind": "constructor",
                    "modifiers": [],
                    "name": "",
                    "nodeType": "FunctionDefinition",
                    "overrides": null,
                    "parameters": {
                      "id": 6,
                      "nodeType": "ParameterList",
                      "parameters": [],
                      "src": "88:2:0"
                    },
                    "returnParameters": {
                      "id": 7,
                      "nodeType": "ParameterList",
                      "parameters": [],
                      "src": "98:0:0"
                    },
                    "scope": 10,
                    "src": "77:26:0",
                    "stateMutability": "nonpayable",
                    "virtual": false,
                    "visibility": "public"
                  }
                ],
                "scope": 11,
                "src": "26:80:0"
              }
            ],
            "src": "0:107:0"
          }"#;

        let source_unit: SourceUnit = serde_json::from_str(json).unwrap();

        let contract = source_unit.nodes[1].as_ref();
        dbg!(&contract);
        println!("{contract:#?}");
    }
}
