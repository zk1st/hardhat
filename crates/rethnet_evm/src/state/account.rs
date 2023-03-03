use revm::primitives::{AccountInfo, Bytecode};

use super::storage::RethnetStorage;

#[derive(Clone, Debug, Default)]
pub struct RethnetAccount {
    pub info: AccountInfo,
    pub storage: RethnetStorage,
}

impl RethnetAccount {
    pub fn new(info: AccountInfo) -> Self {
        Self {
            info,
            storage: RethnetStorage::default(),
        }
    }

    /// Splits the code from the `AccountInfo`, if it exists.
    pub fn split_code(&mut self) -> Option<Bytecode> {
        if let Some(code) = self.info.code.take() {
            if !code.is_empty() {
                self.info.code_hash = code.hash();
                return Some(code);
            }
        }

        None
    }
}
