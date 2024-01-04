use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use edr_eth::Bytes;
use thiserror::Error;
use versions::SemVer;

use super::{
    ast::{self, ContractDefinition, FunctionDefinition, TopLevelNode},
    contract_inheritance::get_flattened_contracts,
    public_functions_map::PublicFunctionsMap,
    standard_json::{CompilerInput, CompilerOutput, CompilerOutputBytecode, CompilerOutputSource},
    utils::{contract_kind_to_contract_type, parse_ast_src_location, ContractData},
};
use crate::model::{
    AnonymousContractFunction, Bytecode, BytecodeType, Codebase, Contract, ContractFunction,
    FreeFunction, Function, ImmutableReference, InternalContractFunction, PublicContractFunction,
    SourceFile,
};

/// Errors that can happen while building the model
#[derive(Error, Debug)]
pub enum ModelBuilderError {
    /// Failed to parse the compiler input
    #[error("Failed to parse the compiler input")]
    CompilerInputParseError,
    /// Failed to parse the compiler output
    #[error("Failed to parse the compiler output")]
    CompilerOutputParseError,
    /// Couldn't find source file in the compilation input
    #[error("Couldn't find source file \"{0}\" in the compilation input")]
    SourceFileNotFound(String),
    /// Couldn't compute the selector for a public function
    #[error("Couldn't compute the selector for {0}:{1}#{2}. This doesn't happen in solc >= 0.6.0 or if you don't use overloads")]
    SelectorError(String, String, String),
    /// Couldn't compute a method identifier
    #[error("Couldn't compute the method identifier for {0}:{1}#{2}.")]
    MethodIdentifierError(String, String, String),
    /// Couldn't find the base contract for a contract
    #[error("Couldn't find base contract {0}")]
    BaseContractNotFound(u32),
    /// Couldn't find base contract's function for a contract
    #[error("Couldn't find base contract's function {0}")]
    BaseContractFunctionNotFound(u32),
    /// Couldn't parse a bytecode
    #[error("Couldn't parse bytecode {0}")]
    BytecodeParseError(String),
}

/// Builds a codebase model from the compiler input and output
pub fn build_model(
    solc_version: SemVer,
    solc_input: &str,
    solc_output: &str,
) -> Result<Codebase, ModelBuilderError> {
    let _compiler_input: CompilerInput = serde_json::from_str(solc_input)
        .map_err(|_err| ModelBuilderError::CompilerInputParseError)?;
    let compiler_output: CompilerOutput = serde_json::from_str(solc_output)
        .map_err(|_err| ModelBuilderError::CompilerOutputParseError)?;

    let public_functions_map = PublicFunctionsMap::from_compiler_output(&compiler_output);

    let mut functions = HashMap::new();
    let mut non_flattened_contracts: VecDeque<(u32, ContractData)> = VecDeque::new();

    for (source_name, compiler_output_source) in &compiler_output.sources {
        add_functions_from_source_file(
            &mut functions,
            source_name,
            compiler_output_source,
            &public_functions_map,
        )?;

        add_non_flattened_contracts(
            &functions,
            &mut non_flattened_contracts,
            compiler_output_source,
        )?;
    }

    let contracts = get_flattened_contracts(&functions, non_flattened_contracts)?;

    let mut source_and_contract_name_to_contract: HashMap<String, HashMap<String, Arc<Contract>>> =
        HashMap::new();
    for (source_name, compiler_output_source) in &compiler_output.sources {
        for node in compiler_output_source.ast.nodes.iter().flatten() {
            if let TopLevelNode::ContractDefinition(contract_definition) = node {
                if let Some(contract) = contracts.get(&contract_definition.id) {
                    source_and_contract_name_to_contract
                        .entry(source_name.clone())
                        .or_default()
                        .insert(contract_definition.name.clone(), contract.clone());
                }
            }
        }
    }

    let mut source_files = HashMap::new();
    for (source_name, compiler_output_source) in &compiler_output.sources {
        add_source_file(
            &mut source_files,
            source_name,
            compiler_output_source,
            &functions,
            &contracts,
        )?;
    }

    // Disabling mutable_key_type because `Bytecode` has a `Bytes` field, which is
    // known to cause false positives for this rule.
    // See https://rust-lang.github.io/rust-clippy/master/index.html#/mutable_key_type
    #[allow(clippy::mutable_key_type)]
    let mut bytecodes = HashSet::new();
    for (source_name, contracts) in &compiler_output.contracts {
        for (contract_name, contract_output) in contracts {
            let contract = source_and_contract_name_to_contract
                .get(source_name)
                .and_then(|contracts| contracts.get(contract_name));

            let contract = if let Some(contract) = contract {
                contract
            } else {
                continue;
            };

            let (normalized_code, library_offsets, immutable_references) =
                normalize_bytecode(&contract_output.evm.deployed_bytecode)?;

            let runtime_bytecode = Bytecode::new(
                Some(contract.clone()),
                BytecodeType::Runtime,
                normalized_code,
                library_offsets,
                immutable_references,
                Some(&contract_output.evm.deployed_bytecode.source_map),
            );

            let (normalized_code, library_offsets, immutable_references) =
                normalize_bytecode(&contract_output.evm.bytecode)?;

            let deployment_bytecode = Bytecode::new(
                Some(contract.clone()),
                BytecodeType::Deployment,
                normalized_code,
                library_offsets,
                immutable_references,
                Some(&contract_output.evm.bytecode.source_map),
            );

            bytecodes.insert(runtime_bytecode);
            bytecodes.insert(deployment_bytecode);
        }
    }

    Ok(Codebase {
        functions,
        contracts,
        source_files,
        bytecodes,
        solc_version,
    })
}

fn add_functions_from_source_file(
    functions: &mut HashMap<u32, Arc<Function>>,
    source_name: &str,
    compiler_output_source: &CompilerOutputSource,
    public_functions_map: &PublicFunctionsMap,
) -> Result<(), ModelBuilderError> {
    for node in &compiler_output_source.ast.nodes {
        let node = if let Some(node) = node {
            node
        } else {
            continue;
        };

        match node {
            TopLevelNode::ContractDefinition(contract_definition) => {
                add_functions_from_contract_definition(
                    functions,
                    source_name.to_string(),
                    contract_definition,
                    public_functions_map,
                )?;
            }
            TopLevelNode::FunctionDefinition(function_definition) => {
                add_function_from_free_function_definition(functions, function_definition);
            }
        }
    }

    Ok(())
}

fn add_non_flattened_contracts(
    functions: &HashMap<u32, Arc<Function>>,
    non_flattened_contracts: &mut VecDeque<(u32, ContractData)>,
    compiler_output_source: &CompilerOutputSource,
) -> Result<(), ModelBuilderError> {
    for node in &compiler_output_source.ast.nodes {
        let node = if let Some(node) = node {
            node
        } else {
            continue;
        };

        if let TopLevelNode::ContractDefinition(contract_definition) = node {
            add_non_flattened_contract(functions, non_flattened_contracts, contract_definition)?;
        }
    }

    Ok(())
}

fn add_non_flattened_contract(
    functions: &HashMap<u32, Arc<Function>>,
    non_flattened_contracts: &mut VecDeque<(u32, ContractData)>,
    contract_definition: &ContractDefinition,
) -> Result<(), ModelBuilderError> {
    let contract_type = contract_kind_to_contract_type(&contract_definition.contract_kind);

    if let Some(contract_type) = contract_type {
        let mut contract_data = ContractData {
            name: contract_definition.name.clone(),
            contract_type,
            location: parse_ast_src_location(&contract_definition.src),
            linearized_base_contract_ids: contract_definition.linearized_base_contracts.clone(),
            local_function_ids: Vec::new(),
            constructor: None,
            fallback: None,
            receive: None,
        };

        contract_definition
            .nodes
            .iter()
            .filter_map(|node| node.as_ref())
            .try_for_each(|node| {
                match node {
                    ast::ContractNode::FunctionDefinition(function_definition) => {
                        contract_data
                            .local_function_ids
                            .push(function_definition.id);

                        if let Some(function) = functions.get(&function_definition.id) {
                            match function.as_ref() {
                                Function::ContractFunction(ContractFunction::Fallback(f)) => {
                                    contract_data.fallback = Some(f.clone());
                                }
                                Function::ContractFunction(ContractFunction::Receive(f)) => {
                                    contract_data.receive = Some(f.clone());
                                }
                                Function::ContractFunction(ContractFunction::Constructor(f)) => {
                                    contract_data.constructor = Some(f.clone());
                                }
                                _ => {}
                            }
                        }
                    }
                    ast::ContractNode::ModifierDefinition(modifier_definition) => {
                        contract_data
                            .local_function_ids
                            .push(modifier_definition.id);
                    }
                    ast::ContractNode::VariableDeclaration(variable_declaration) => {
                        if variable_declaration.visibility == ast::Visibility::Public {
                            contract_data
                                .local_function_ids
                                .push(variable_declaration.id);
                        }
                    }
                };

                Ok(())
            })?;

        non_flattened_contracts.push_back((contract_definition.id, contract_data));
    }

    Ok(())
}

fn add_functions_from_contract_definition(
    functions: &mut HashMap<u32, Arc<Function>>,
    source_name: String,
    contract_definition: &ContractDefinition,
    public_functions_map: &PublicFunctionsMap,
) -> Result<(), ModelBuilderError> {
    for node in &contract_definition.nodes {
        let node = if let Some(node) = node {
            node
        } else {
            continue;
        };

        match node {
            ast::ContractNode::FunctionDefinition(function_definition) => {
                add_function_from_contract_function_definition(
                    functions,
                    function_definition,
                    source_name.clone(),
                    contract_definition.name.clone(),
                    public_functions_map,
                )?;
            }
            ast::ContractNode::ModifierDefinition(modifier_definition) => {
                add_function_from_modifier_definition(functions, modifier_definition);
            }
            ast::ContractNode::VariableDeclaration(variable_declaration) => {
                if variable_declaration.visibility == ast::Visibility::Public {
                    add_function_from_public_variable_declaration(
                        functions,
                        public_functions_map,
                        &source_name,
                        &contract_definition.name,
                        variable_declaration,
                    )?;
                }
            }
        }
    }

    Ok(())
}

fn add_function_from_free_function_definition(
    functions: &mut HashMap<u32, Arc<Function>>,
    function_definition: &FunctionDefinition,
) {
    functions.insert(
        function_definition.id,
        Arc::new(Function::FreeFunction(FreeFunction {
            name: function_definition.name.clone(),
            location: parse_ast_src_location(&function_definition.src),
        })),
    );
}

fn add_function_from_contract_function_definition(
    functions: &mut HashMap<u32, Arc<Function>>,
    function_definition: &FunctionDefinition,
    source_name: String,
    contract_name: String,
    public_functions_map: &PublicFunctionsMap,
) -> Result<(), ModelBuilderError> {
    let function = match function_definition.kind {
        ast::FunctionKind::Function => match function_definition.visibility {
            ast::Visibility::External | ast::Visibility::Public => {
                let selector = try_build_public_contract_function_selector(
                    public_functions_map,
                    &source_name,
                    &contract_name,
                    &function_definition.name,
                    Some(function_definition.parameters.parameters.len()),
                    function_definition.function_selector.clone(),
                )?;

                let method_identifier = try_build_public_contract_function_method_identifier(
                    public_functions_map,
                    &source_name,
                    &contract_name,
                    &function_definition.name,
                    selector,
                )?;
                ContractFunction::PublicFunction(Arc::new(
                    try_build_public_contract_function_model(
                        &function_definition.src,
                        &function_definition.name,
                        selector,
                        &method_identifier,
                        function_definition.state_mutability == ast::StateMutability::Payable,
                    )?,
                ))
            }
            ast::Visibility::Internal | ast::Visibility::Private => {
                ContractFunction::InternalFunction(build_internal_contract_function_model(
                    function_definition,
                ))
            }
        },
        ast::FunctionKind::Receive => ContractFunction::Receive(Arc::new(
            build_anonymous_contract_function_model(function_definition),
        )),
        ast::FunctionKind::Constructor => ContractFunction::Constructor(Arc::new(
            build_anonymous_contract_function_model(function_definition),
        )),
        ast::FunctionKind::Fallback => ContractFunction::Fallback(Arc::new(
            build_anonymous_contract_function_model(function_definition),
        )),
        ast::FunctionKind::FreeFunction => {
            unreachable!("Free functions should not be processed as contract functions")
        }
    };

    functions.insert(
        function_definition.id,
        Arc::new(Function::ContractFunction(function)),
    );

    Ok(())
}

fn build_internal_contract_function_model(
    function_definition: &FunctionDefinition,
) -> InternalContractFunction {
    InternalContractFunction {
        name: function_definition.name.clone(),
        location: parse_ast_src_location(&function_definition.src),
    }
}

fn build_anonymous_contract_function_model(
    function_definition: &FunctionDefinition,
) -> AnonymousContractFunction {
    AnonymousContractFunction {
        location: parse_ast_src_location(&function_definition.src),
        public: function_definition.visibility == ast::Visibility::Public
            || function_definition.visibility == ast::Visibility::External,
    }
}

fn try_build_public_contract_function_selector(
    public_functions_map: &PublicFunctionsMap,
    source_name: &str,
    contract_name: &str,
    function_name: &str,
    parameters_count: Option<usize>,
    hex_selector: Option<String>,
) -> Result<[u8; 4], ModelBuilderError> {
    public_functions_map
        .get_function_selector(
            source_name,
            contract_name,
            function_name,
            parameters_count,
            hex_selector,
        )
        .ok_or(ModelBuilderError::SelectorError(
            source_name.to_string(),
            function_name.to_string(),
            contract_name.to_string(),
        ))
}

fn try_build_public_contract_function_method_identifier(
    public_functions_map: &PublicFunctionsMap,
    source_name: &str,
    contract_name: &str,
    function_name: &str,
    selector: [u8; 4],
) -> Result<String, ModelBuilderError> {
    public_functions_map
        .get_method_identifier(source_name, contract_name, function_name, &selector)
        .ok_or(ModelBuilderError::MethodIdentifierError(
            source_name.to_string(),
            function_name.to_string(),
            contract_name.to_string(),
        ))
}

fn try_build_public_contract_function_model(
    src: &str,
    function_name: &str,
    selector: [u8; 4],
    method_identifier: &str,
    payable: bool,
) -> Result<PublicContractFunction, ModelBuilderError> {
    Ok(PublicContractFunction {
        name: function_name.to_string(),
        location: parse_ast_src_location(src),
        payable,
        selector,
        method_identifier: method_identifier.to_string(),
    })
}

fn add_function_from_modifier_definition(
    functions: &mut HashMap<u32, Arc<Function>>,
    modifier_definition: &ast::ModifierDefinition,
) {
    functions.insert(
        modifier_definition.id,
        Arc::new(Function::ContractFunction(ContractFunction::Modifier(
            InternalContractFunction {
                name: modifier_definition.name.clone(),
                location: parse_ast_src_location(&modifier_definition.src),
            },
        ))),
    );
}

fn add_function_from_public_variable_declaration(
    functions: &mut HashMap<u32, Arc<Function>>,
    public_functions_map: &PublicFunctionsMap,
    source_name: &str,
    contract_name: &str,
    variable_declaration: &ast::VariableDeclaration,
) -> Result<(), ModelBuilderError> {
    let selector = try_build_public_contract_function_selector(
        public_functions_map,
        source_name,
        contract_name,
        &variable_declaration.name,
        None,
        None,
    )?;

    let method_identifier = try_build_public_contract_function_method_identifier(
        public_functions_map,
        source_name,
        contract_name,
        &variable_declaration.name,
        selector,
    )?;

    functions.insert(
        variable_declaration.id,
        Arc::new(Function::ContractFunction(ContractFunction::Getter(
            Arc::new(try_build_public_contract_function_model(
                &variable_declaration.src,
                &variable_declaration.name,
                selector,
                &method_identifier,
                false,
            )?),
        ))),
    );

    Ok(())
}

fn add_source_file(
    source_files: &mut HashMap<u32, SourceFile>,
    source_name: &str,
    compiler_output_source: &CompilerOutputSource,
    functions: &HashMap<u32, Arc<Function>>,
    contracts: &HashMap<u32, Arc<Contract>>,
) -> Result<(), ModelBuilderError> {
    let mut source_file = SourceFile::new(source_name.to_string());

    for node in &compiler_output_source.ast.nodes {
        let node = if let Some(node) = node {
            node
        } else {
            continue;
        };

        match node {
            TopLevelNode::ContractDefinition(contract_definition) => {
                if let Some(contract) = contracts.get(&contract_definition.id) {
                    let source_location = parse_ast_src_location(&contract_definition.src);
                    source_file
                        .contracts
                        .insert(source_location, contract.clone());
                }
            }
            TopLevelNode::FunctionDefinition(function_definition) => {
                if let Some(function) = functions.get(&function_definition.id) {
                    let source_location = parse_ast_src_location(&function_definition.src);
                    source_file
                        .functions
                        .insert(source_location, function.clone());
                }
            }
        }
    }

    source_files.insert(compiler_output_source.id, source_file);

    Ok(())
}

fn normalize_bytecode(
    compiler_output_bytecode: &CompilerOutputBytecode,
) -> Result<(Bytes, Vec<usize>, Vec<ImmutableReference>), ModelBuilderError> {
    let mut bytecode = compiler_output_bytecode.object.clone();
    let link_references = &compiler_output_bytecode.link_references;

    let library_offsets: Vec<usize> = link_references
        .values()
        .flat_map(|link_reference| {
            link_reference.values().flat_map(|link_reference_offsets| {
                link_reference_offsets
                    .iter()
                    .map(|link_reference_offset| link_reference_offset.start)
            })
        })
        .collect();

    for offset in &library_offsets {
        let hex_offset = 2 * offset;
        bytecode.replace_range(hex_offset..hex_offset + 40, "0".repeat(40).as_str());
    }

    let normalized_bytecode =
        hex::decode(&bytecode).map_err(|_err| ModelBuilderError::BytecodeParseError(bytecode))?;

    let immutable_references: Vec<ImmutableReference> = compiler_output_bytecode
        .immutable_references
        .as_ref()
        .map(|immutable_references| {
            immutable_references
                .values()
                .flat_map(|immutable_references| {
                    immutable_references
                        .iter()
                        .map(|immutable_reference| ImmutableReference {
                            offset: immutable_reference.start,
                            length: immutable_reference.length,
                        })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok((
        normalized_bytecode.into(),
        library_offsets,
        immutable_references,
    ))
}
