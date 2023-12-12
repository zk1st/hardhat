use std::collections::{HashMap, HashSet};

use thiserror::Error;

use super::{
    ast::{self, ContractDefinition, FunctionDefinition, TopLevelNode},
    contract_inheritance::flatten_contract_inheritance,
    public_functions_map::PublicFunctionsMap,
    standard_json::{CompilerInput, CompilerOutput, CompilerOutputSource},
    utils::{contract_kind_to_contract_type, parse_ast_src_location},
};
use crate::model::{
    codebase::Codebase,
    contract::Contract,
    custom_error::CustomError,
    function::{
        AnonymousContractFunction, ContractFunction, FreeFunction, Function,
        InternalContractFunction, PublicContractFunction,
    },
    source_file::SourceFile,
};

#[derive(Error, Debug)]
pub enum ModelBuilderError {
    #[error("Failed to parse the compiler input")]
    CompilerInputParseError,
    #[error("Failed to parse the compiler output")]
    CompilerOutputParseError,
    #[error("Couldn't find source file \"{0}\" in the compilation input")]
    SourceFileNotFound(String),
    #[error("Couldn't compute the selector for {0}:{1}#{2}. This doesn't happen in solc >= 0.6.0 or if you don't use overloads")]
    SelectorError(String, String, String),
    #[error("Couldn't compute the method identifier for {0}:{1}#{2}.")]
    MethodIdentifierError(String, String, String),
    #[error("Couldn't find base contract {0}")]
    BaseContractNotFound(u32),
    #[error("Couldn't find base contract's function {0}")]
    BaseContractFunctionNotFound(u32),
}

#[derive(Default, Debug)]
pub(crate) struct BuilderState {
    pub source_files: HashMap<u32, SourceFile>,
    pub contracts: HashMap<u32, Contract>,
    pub functions: HashMap<u32, Function>,
    pub custom_errors: HashMap<u32, CustomError>,
}

pub fn build_model(solc_input: &str, solc_output: &str) -> Result<Codebase, ModelBuilderError> {
    let compiler_input: CompilerInput = serde_json::from_str(solc_input)
        .map_err(|_err| ModelBuilderError::CompilerInputParseError)?;
    let compiler_output: CompilerOutput = serde_json::from_str(solc_output)
        .map_err(|_err| ModelBuilderError::CompilerOutputParseError)?;

    let public_functions_map = PublicFunctionsMap::from_compiler_output(&compiler_output);

    let mut builder_state = BuilderState::default();

    compiler_output
        .sources
        .iter()
        .try_for_each(|(source_name, output_source)| {
            let source_file_content = compiler_input
                .sources
                .get(source_name)
                .ok_or(ModelBuilderError::SourceFileNotFound(source_name.clone()))?
                .content
                .clone();

            process_ast_source_file(
                &mut builder_state,
                &public_functions_map,
                source_name,
                &source_file_content,
                output_source,
            )?;

            Ok(())
        })?;

    // Process ABIs
    //   Process custom errors

    flatten_contract_inheritance(&mut builder_state)?;

    // decode bytecodes

    Ok(Codebase {
        source_files: builder_state.source_files,
        contracts: builder_state.contracts,
        functions: builder_state.functions,
        custom_errors: builder_state.custom_errors,
        bytecodes: Vec::default(),
    })
}

fn process_ast_source_file(
    builder_state: &mut BuilderState,
    pubic_functions_map: &PublicFunctionsMap,
    source_name: &str,
    source_file_content: &str,
    output_source: &CompilerOutputSource,
) -> Result<(), ModelBuilderError> {
    let source_file = SourceFile::new(source_name.to_string(), source_file_content.to_string());

    output_source
        .ast
        .nodes
        .iter()
        .filter_map(|node| node.as_ref())
        .try_for_each(|node| {
            match node {
                TopLevelNode::ContractDefinition(contract_defintion) => {
                    process_contract_definition(
                        builder_state,
                        pubic_functions_map,
                        &source_file.source_name,
                        contract_defintion,
                    )?;
                }
                TopLevelNode::FunctionDefinition(free_function) => {
                    process_free_function_definition(builder_state, free_function)?;
                }
            };

            Ok(())
        })?;

    builder_state
        .source_files
        .insert(output_source.id, source_file);

    Ok(())
}

fn process_contract_definition(
    builder_state: &mut BuilderState,
    pubic_functions_map: &PublicFunctionsMap,
    source_name: &String,
    contract_defintion: &ContractDefinition,
) -> Result<(), ModelBuilderError> {
    let contract_type = contract_kind_to_contract_type(&contract_defintion.contract_kind);

    if let Some(contract_type) = contract_type {
        let mut contract = Contract::new(
            contract_defintion.name.clone(),
            contract_type,
            parse_ast_src_location(&contract_defintion.src),
            contract_defintion.linearized_base_contracts.clone(),
        );

        contract_defintion
            .nodes
            .iter()
            .filter_map(|node| node.as_ref())
            .try_for_each(|node| {
                match node {
                    ast::ContractNode::FunctionDefinition(function_definition) => {
                        contract.local_function_ids.insert(function_definition.id);

                        process_contract_function_definition(
                            builder_state,
                            pubic_functions_map,
                            source_name,
                            &mut contract,
                            contract_defintion.id,
                            function_definition,
                        )?;
                    }
                    ast::ContractNode::ModifierDefinition(modifier_definition) => {
                        contract.local_function_ids.insert(modifier_definition.id);

                        process_modifier_definition(
                            builder_state,
                            contract_defintion.id,
                            modifier_definition,
                        )?;
                    }
                    ast::ContractNode::VariableDeclaration(variable_declaration) => {
                        if variable_declaration.visibility == ast::Visibility::Public {
                            contract.local_function_ids.insert(variable_declaration.id);

                            process_public_variable_declaration(
                                builder_state,
                                pubic_functions_map,
                                source_name,
                                &contract.name,
                                contract_defintion.id,
                                variable_declaration,
                            )?;
                        }
                    }
                };

                Ok(())
            })?;

        builder_state
            .contracts
            .insert(contract_defintion.id, contract);
    }

    Ok(())
}

fn process_modifier_definition(
    builder_state: &mut BuilderState,
    contract_id: u32,
    modifier_definition: &ast::ModifierDefinition,
) -> Result<(), ModelBuilderError> {
    builder_state.functions.insert(
        modifier_definition.id,
        Function::ContractFunction(ContractFunction::Modifier(InternalContractFunction {
            name: modifier_definition.name.clone(),
            location: parse_ast_src_location(&modifier_definition.src),
            contract_id,
        })),
    );

    Ok(())
}

fn process_free_function_definition(
    builder_state: &mut BuilderState,
    function_definition: &ast::FunctionDefinition,
) -> Result<(), ModelBuilderError> {
    builder_state.functions.insert(
        function_definition.id,
        Function::FreeFunction(FreeFunction {
            name: function_definition.name.clone(),
            location: parse_ast_src_location(&function_definition.src),
        }),
    );

    Ok(())
}

fn process_contract_function_definition(
    builder_state: &mut BuilderState,
    public_functions_map: &PublicFunctionsMap,
    source_name: &String,
    contract: &mut Contract,
    contract_id: u32,
    function_definition: &FunctionDefinition,
) -> Result<(), ModelBuilderError> {
    let function = match function_definition.kind {
        ast::FunctionKind::Function => match function_definition.visibility {
            ast::Visibility::External | ast::Visibility::Public => {
                ContractFunction::PublicFunction(try_build_public_contract_function_model(
                    public_functions_map,
                    source_name,
                    &contract.name,
                    contract_id,
                    &function_definition.src,
                    &function_definition.name,
                    Some(function_definition.parameters.parameters.len()),
                    function_definition.function_selector.as_ref(),
                    function_definition.state_mutability == ast::StateMutability::Payable,
                )?)
            }
            ast::Visibility::Internal | ast::Visibility::Private => {
                ContractFunction::InternalFunction(build_internal_contract_function_model(
                    contract_id,
                    function_definition,
                ))
            }
        },
        ast::FunctionKind::Receive => ContractFunction::Receive(
            build_anonymous_contract_function_model(contract_id, function_definition),
        ),
        ast::FunctionKind::Constructor => ContractFunction::Constructor(
            build_anonymous_contract_function_model(contract_id, function_definition),
        ),
        ast::FunctionKind::Fallback => ContractFunction::Fallback(
            build_anonymous_contract_function_model(contract_id, function_definition),
        ),
        ast::FunctionKind::FreeFunction => {
            unreachable!("Free functions should not be processed as contract functions")
        }
    };

    match function {
        ContractFunction::Fallback(_) => contract.fallback_id = Some(function_definition.id),
        ContractFunction::Receive(_) => contract.receive_id = Some(function_definition.id),
        ContractFunction::Constructor(_) => contract.constructor_id = Some(function_definition.id),
        _ => {}
    }

    builder_state
        .functions
        .insert(function_definition.id, Function::ContractFunction(function));

    Ok(())
}

fn process_public_variable_declaration(
    builder_state: &mut BuilderState,
    public_functions_map: &PublicFunctionsMap,
    source_name: &String,
    contract_name: &String,
    contract_id: u32,
    variable_declaration: &ast::VariableDeclaration,
) -> Result<(), ModelBuilderError> {
    builder_state.functions.insert(
        variable_declaration.id,
        Function::ContractFunction(ContractFunction::Getter(
            try_build_public_contract_function_model(
                public_functions_map,
                source_name,
                contract_name,
                contract_id,
                &variable_declaration.src,
                &variable_declaration.name,
                None,
                None,
                false,
            )?,
        )),
    );

    Ok(())
}

fn process_internal_contract_function_definition(
    builder_state: &mut BuilderState,
    contract_id: u32,
    function_definition: &ast::FunctionDefinition,
) -> Result<(), ModelBuilderError> {
    builder_state.functions.insert(
        function_definition.id,
        Function::ContractFunction(ContractFunction::InternalFunction(
            InternalContractFunction {
                name: function_definition.name.clone(),
                location: parse_ast_src_location(&function_definition.src),
                contract_id,
            },
        )),
    );

    Ok(())
}

fn build_anonymous_contract_function_model(
    contract_id: u32,
    function_definition: &FunctionDefinition,
) -> AnonymousContractFunction {
    AnonymousContractFunction {
        name: function_definition.name.clone(),
        location: parse_ast_src_location(&function_definition.src),
        contract_id,
        is_public: function_definition.visibility == ast::Visibility::Public
            || function_definition.visibility == ast::Visibility::External,
    }
}

fn build_internal_contract_function_model(
    contract_id: u32,
    function_definition: &FunctionDefinition,
) -> InternalContractFunction {
    InternalContractFunction {
        name: function_definition.name.clone(),
        location: parse_ast_src_location(&function_definition.src),
        contract_id,
    }
}

fn try_build_public_contract_function_model(
    public_functions_map: &PublicFunctionsMap,
    source_name: &String,
    contract_name: &String,
    contract_id: u32,
    src: &ast::SourceLocation,
    function_name: &String,
    parameters_count: Option<usize>,
    hex_selector: Option<&String>,
    payable: bool,
) -> Result<PublicContractFunction, ModelBuilderError> {
    let function_name = function_name.clone();

    let selector = public_functions_map
        .get_function_selector(
            source_name,
            contract_name,
            &function_name,
            parameters_count,
            hex_selector,
        )
        .ok_or(ModelBuilderError::SelectorError(
            source_name.clone(),
            function_name.clone(),
            contract_name.clone(),
        ))?;

    let method_identifier = public_functions_map
        .get_method_identifier(source_name, contract_name, &function_name, &selector)
        .ok_or(ModelBuilderError::MethodIdentifierError(
            source_name.clone(),
            function_name.clone(),
            contract_name.clone(),
        ))?;

    Ok(PublicContractFunction {
        name: function_name,
        location: parse_ast_src_location(src),
        contract_id,
        payable,
        selector,
        method_identifier,
    })
}
