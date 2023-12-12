use std::collections::HashSet;

#[derive(Debug)]
pub struct SourceFile {
    pub source_name: String,
    pub content: String,
    pub contract_ids: HashSet<usize>,
    pub global_function_ids: HashSet<usize>,

    // Are they included in the contracts that use them?
    pub global_custom_error_ids: HashSet<usize>,
}

impl SourceFile {
    pub fn new(source_name: String, content: String) -> Self {
        Self {
            source_name,
            content: "".to_string(),
            contract_ids: HashSet::new(),
            global_function_ids: HashSet::new(),
            global_custom_error_ids: HashSet::new(),
        }
    }
}
