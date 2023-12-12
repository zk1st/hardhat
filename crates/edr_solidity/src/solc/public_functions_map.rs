use std::collections::{HashMap, HashSet};

use super::{standard_json::CompilerOutput, utils::parse_hex_function_selector};

/// This map is used to get the selector and method identifier of a function.
///
/// This could be simpler, but solc 0.5.x selectors are not available in the
/// AST, and computing them based on the AST is too complex.
///
/// Whenever a function selector is available in the AST, we use it, and this
/// is completely robust.
///
/// When it's not available, we try our best to fetch one.
///
/// When overloads are not present, this is as robust as having the selector.
/// When a function is overloaded, we try to get the right selector
/// based on the number of parameters.
///
/// Also, we don't know the number of parameters that an automatically created
/// getter has, so we only support them if they are not overloaded.
pub struct PublicFunctionsMap {
    // source name -> contract name -> function name -> PublicFunctionMapEntry
    inner: HashMap<String, HashMap<String, HashMap<String, PublicFunctionMapEntry>>>,
}

#[derive(Default)]
struct PublicFunctionMapEntry {
    selectors_by_param_count: HashMap<usize, HashSet<[u8; 4]>>,
    method_identifier_by_selector: HashMap<[u8; 4], String>,
}

impl PublicFunctionsMap {
    pub fn from_compiler_output(compiler_output: &CompilerOutput) -> Self {
        PublicFunctionsMap {
            inner: compiler_output
                .contracts
                .iter()
                .map(|(source_name, contracts)| {
                    let contracts = contracts
                        .iter()
                        .map(|(contract_name, contract)| {
                            let mut functions = HashMap::default();

                            contract.evm.method_identifiers.iter().for_each(
                                |(method_identifier, selector)| {
                                    let params_count =
                                        method_identifier.chars().filter(|c| *c == ',').count() + 1;

                                    let function_name = method_identifier
                                        .split('(')
                                        .next()
                                        .expect("Invalid method identifier");

                                    let selector = parse_hex_function_selector(selector)
                                        .expect("Invalid selector in the compilation output");

                                    let entry = functions
                                        .entry(function_name.to_string())
                                        .or_insert_with(PublicFunctionMapEntry::default);

                                    entry
                                        .selectors_by_param_count
                                        .entry(params_count)
                                        .or_insert_with(HashSet::default)
                                        .insert(selector);

                                    entry
                                        .method_identifier_by_selector
                                        .insert(selector, method_identifier.clone());
                                },
                            );

                            (contract_name.clone(), functions)
                        })
                        .collect();

                    (source_name.clone(), contracts)
                })
                .collect(),
        }
    }

    /// Returns the selector of a function if possible.
    ///
    /// If the number of parameters is None, it will return the selector if the
    /// function is not overloaded.
    pub fn get_function_selector(
        &self,
        source_name: &String,
        contract_name: &String,
        function_name: &String,
        parameters_count: Option<usize>,
        hex_selector: Option<&String>,
    ) -> Option<[u8; 4]> {
        hex_selector
            .and_then(parse_hex_function_selector)
            .or_else(|| {
                self.inner
                    .get(source_name)
                    .and_then(|contracts| contracts.get(contract_name))
                    .and_then(|functions| functions.get(function_name))
                    .and_then(|entry| {
                        parameters_count.map_or_else(
                            || {
                                if entry.selectors_by_param_count.len() == 1 {
                                    entry.selectors_by_param_count.values().next()
                                } else {
                                    None
                                }
                            },
                            |count| entry.selectors_by_param_count.get(&count),
                        )
                    })
                    .and_then(|selectors| {
                        if selectors.len() == 1 {
                            selectors.iter().next().cloned()
                        } else {
                            None
                        }
                    })
            })
    }

    /// Returns the method identifier of a function if possible.
    pub fn get_method_identifier(
        &self,
        source_name: &String,
        contract_name: &String,
        function_name: &String,
        selector: &[u8; 4],
    ) -> Option<String> {
        self.inner
            .get(source_name)
            .and_then(|contracts| contracts.get(contract_name))
            .and_then(|functions| functions.get(function_name))
            .and_then(|entry| entry.method_identifier_by_selector.get(selector))
            .cloned()
    }
}
