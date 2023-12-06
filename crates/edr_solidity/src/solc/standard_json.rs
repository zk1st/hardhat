use std::collections::HashMap;

use serde::Deserialize;

use super::ast::SourceUnit;

#[derive(Debug, Deserialize)]
pub struct CompilerInputSourceFile {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CompilerInput {
    pub sources: HashMap<String, CompilerInputSourceFile>,
}

pub struct CompilerOutput {
    sources: HashMap<String, CompilerOutputSource>,
    // Sourcename to contractname to contract
    contracts: HashMap<String, HashMap<String, CompilerOutputContract>>,
}

#[derive(Debug, Deserialize)]
pub struct CompilerOutputSource {
    pub id: u32,
    pub ast: SourceUnit,
}

#[derive(Debug, Deserialize)]
pub struct CompilerOutputContract {
    // abi: any;
    pub evm: CompilerOutputContractEvm,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilerOutputContractEvm {
    pub bytecode: CompilerOutputBytecode,
    pub deployed_bytecode: CompilerOutputBytecode,
    // Method signature to method identifier
    pub method_identifiers: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilerOutputBytecode {
    pub object: String,
    pub opcodes: String,
    pub source_map: String,
    pub link_references: HashMap<String, HashMap<String, Vec<HexBytecodeSlice>>>,
    pub immutable_references: Option<HashMap<String, Vec<HexBytecodeSlice>>>,
}

#[derive(Debug, Deserialize)]
pub struct HexBytecodeSlice {
    pub start: usize,
    pub length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_extra_fields() {
        let data = r#"{
        "sources": {
        },
        "foo": "bar"
    }"#;

        let compiler_input: CompilerInput = serde_json::from_str(data).unwrap();
        assert_eq!(compiler_input.sources.len(), 0);
    }

    #[test]
    fn test_deserialize_input() {
        let data = r#"{
          "sources": {
            "test.sol": {
              "content": "input1"
            },
            "foo/test2.sol": {
              "content": "input2"
            }
          }
      }"#;

        let compiler_input: CompilerInput = serde_json::from_str(data).unwrap();
        assert_eq!(compiler_input.sources.len(), 2);

        assert_eq!(
            compiler_input.sources.get("test.sol").unwrap().content,
            "input1"
        );
        assert_eq!(
            compiler_input.sources.get("foo/test2.sol").unwrap().content,
            "input2"
        );
    }

    fn test_deserialize_output() {}
}
