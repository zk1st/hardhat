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

#[derive(Debug, Deserialize)]
pub struct CompilerOutput {
    pub sources: HashMap<String, CompilerOutputSource>,
    // Sourcename to contractname to contract
    pub contracts: HashMap<String, HashMap<String, CompilerOutputContract>>,
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
    use std::{
        fs, io,
        path::{Path, PathBuf},
    };

    use versions::SemVer;

    use super::*;
    use crate::solc::model_builder::build_model;

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

    #[test]
    fn test_deserialize_output() {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let artifacts_path = crate_root.join("../../packages/hardhat-core/test/internal/hardhat-network/stack-traces/test-files/artifacts");

        assert!(
            artifacts_path.exists(),
            "artifacts path doesn't exist: {:?}",
            artifacts_path
        );

        visit_dirs(artifacts_path.as_path(), &|path| {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let path = path.to_str().unwrap();

            if file_name.starts_with("compiler-output") && file_name.ends_with(".json") {
                println!("Testing {path}\n\n\n\n\n");
                let input_content = fs::read_to_string(path.replace("output", "input")).unwrap();
                let output_content = fs::read_to_string(path).unwrap();

                let raw_version = path.split("/").last().unwrap().split("-").nth(3).unwrap();

                let solc_version = SemVer::parse(raw_version).unwrap().1;

                let codebase = build_model(solc_version, &input_content, &output_content).unwrap();

                print!("{:#?}\n\n\n\n\n\n\n", &codebase);
            }
        })
        .unwrap();
    }

    #[test]
    fn test() {
        println!(
            "{:#?}",
            alloy_json_abi::Function::parse("i()").unwrap().selector()
        );

        println!(
            "{:#?}",
            alloy_json_abi::Function::parse("sss(address,address,uint256,uint256)")
                .unwrap()
                .selector()
        );

        let custom_error: alloy_json_abi::Error = serde_json::from_str(
            r#"{
			"inputs": [
				{
					"internalType": "int256",
					"name": "i",
					"type": "int256"
				},
				{
					"internalType": "string",
					"name": "asd",
					"type": "string"
				}
			],
			"name": "Foo",
			"type": "error"
		}"#,
        )
        .unwrap();

        println!("{:#?}", custom_error);
    }

    fn visit_dirs<F>(dir: &Path, file_operation: &F) -> io::Result<()>
    where
        F: Fn(&PathBuf),
    {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, file_operation)?;
                } else {
                    file_operation(&path);
                }
            }
        }
        Ok(())
    }
}
