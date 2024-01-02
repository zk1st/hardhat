use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use super::{model_builder::ModelBuilderError, utils::ContractData};
use crate::model::{Contract, ContractFunction, Function};

fn build_basic_flattened(
    functions: &HashMap<u32, Arc<Function>>,
    contract_data: &ContractData,
) -> Result<Contract, ModelBuilderError> {
    let mut selector_to_function = HashMap::default();

    contract_data
        .local_function_ids
        .iter()
        .try_for_each(|function_id| {
            let function = functions.get(function_id).ok_or(
                ModelBuilderError::BaseContractFunctionNotFound(*function_id),
            )?;

            match function.as_ref() {
                Function::ContractFunction(contract_function) => match contract_function {
                    ContractFunction::PublicFunction(function)
                    | ContractFunction::Getter(function) => {
                        selector_to_function.insert(function.selector, function.clone());
                    }
                    _ => {}
                },
                Function::FreeFunction(_) => {
                    unreachable!("Only contract functions should be in the local function ids")
                }
            }

            Ok(())
        })?;

    Ok(Contract {
        name: contract_data.name.clone(),
        contract_type: contract_data.contract_type,
        location: contract_data.location,
        custom_errors: Vec::new(),
        constructor: contract_data.constructor.clone(),
        fallback: contract_data.fallback.clone(),
        receive: contract_data.receive.clone(),
        selector_to_function,
    })
}

pub(crate) fn get_flattened_contracts(
    functions: &HashMap<u32, Arc<Function>>,
    mut non_flattened_contracts: VecDeque<(u32, ContractData)>,
) -> Result<HashMap<u32, Arc<Contract>>, ModelBuilderError> {
    let mut result: HashMap<u32, Contract> = HashMap::default();

    while let Some((contract_id, contract_data)) = non_flattened_contracts.pop_front() {
        // Base case: contracts without base contracts
        if contract_data.linearized_base_contract_ids.len() == 1 {
            result.insert(
                contract_id,
                build_basic_flattened(functions, &contract_data)?,
            );
        } else {
            let last_base_id = contract_data.linearized_base_contract_ids.get(1).unwrap();
            let base_flattened = result.get(last_base_id);

            if let Some(base_flattened) = base_flattened {
                let mut flattened = build_basic_flattened(functions, &contract_data)?;

                if base_flattened.constructor.is_none() {
                    flattened.constructor = base_flattened.constructor.clone();
                }

                if base_flattened.receive.is_none() {
                    flattened.receive = base_flattened.receive.clone();
                }

                if base_flattened.fallback.is_none() {
                    flattened.fallback = base_flattened.fallback.clone();
                }

                for (selector, function_id) in base_flattened.selector_to_function.iter() {
                    if !flattened.selector_to_function.contains_key(selector) {
                        flattened
                            .selector_to_function
                            .insert(*selector, function_id.clone());
                    }
                }

                result.insert(contract_id, flattened);
            } else {
                non_flattened_contracts.push_back((contract_id, contract_data));
            };
        }
    }

    Ok(result
        .into_iter()
        .map(|(id, contract)| (id, Arc::new(contract)))
        .collect())
}
