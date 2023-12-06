pub struct SourceFile {
    pub source_name: String,
    pub content: String,
    pub contract_ids: Vec<usize>,
    pub global_function_ids: Vec<usize>,

    // Are they included in the contracts that use them?
    pub global_custom_error_ids: Vec<usize>,
}
