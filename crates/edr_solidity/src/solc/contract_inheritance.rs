use std::collections::{HashMap, VecDeque};

use super::model_builder::{BuilderState, ModelBuilderError};
use crate::model::Contract;

#[derive(Default)]
struct Flattened {
    pub constructor_id: Option<u32>,
    pub fallback_id: Option<u32>,
    pub receive_id: Option<u32>,
    pub selector_to_function_id: HashMap<[u8; 4], u32>,
}

fn build_basic_flattened(
    builder_state: &BuilderState,
    contract: &Contract,
) -> Result<Flattened, ModelBuilderError> {
    let mut selector_to_function_id = HashMap::default();

    contract
        .local_function_ids
        .iter()
        .try_for_each(|function_id| {
            let function = builder_state.functions.get(function_id).ok_or(
                ModelBuilderError::BaseContractFunctionNotFound(*function_id),
            )?;

            match function {
                Function::ContractFunction(contract_functions) => match contract_functions {
                    ContractFunction::PublicFunction(function)
                    | ContractFunction::Getter(function) => {
                        selector_to_function_id.insert(function.selector.clone(), *function_id);
                    }
                    _ => {}
                },
                _ => {
                    unreachable!(
                        "Only contract functions should be in the
local function ids"
                    )
                }
            }

            Ok(())
        })?;

    Ok(Flattened {
        constructor_id: contract.constructor_id,
        fallback_id: contract.fallback_id,
        receive_id: contract.receive_id,
        selector_to_function_id,
    })
}

pub(crate) fn flatten_contract_inheritance(
    builder_state: &mut BuilderState,
) -> Result<(), ModelBuilderError> {
    let mut processed: HashMap<u32, Flattened> = HashMap::default();
    let mut queue: VecDeque<u32> = builder_state.contracts.keys().cloned().collect();

    while let Some(contract_id) = queue.pop_front() {
        let contract = builder_state
            .contracts
            .get(&contract_id)
            .ok_or(ModelBuilderError::BaseContractNotFound(contract_id))?;

        // Base case: contracts without base contracts
        if contract.linearized_base_contract_ids.len() == 1 {
            processed.insert(contract_id, build_basic_flattened(builder_state, contract)?);
        } else {
            let last_base_id = contract.linearized_base_contract_ids.get(1).unwrap();
            let base_flattened = processed.get(last_base_id);

            if let Some(base_flattened) = base_flattened {
                let mut flattened = build_basic_flattened(builder_state, contract)?;

                if base_flattened.constructor_id.is_none() {
                    flattened.constructor_id = base_flattened.constructor_id;
                }

                if base_flattened.receive_id.is_none() {
                    flattened.receive_id = base_flattened.receive_id;
                }

                if base_flattened.fallback_id.is_none() {
                    flattened.fallback_id = base_flattened.fallback_id;
                }

                for (selector, function_id) in base_flattened.selector_to_function_id.iter() {
                    if !flattened.selector_to_function_id.contains_key(selector) {
                        flattened
                            .selector_to_function_id
                            .insert(selector.clone(), *function_id);
                    }
                }

                processed.insert(contract_id, flattened);
            } else {
                queue.push_back(contract_id);
            };
        }
    }

    Ok(())
}
