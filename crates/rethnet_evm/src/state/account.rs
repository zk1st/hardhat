use revm::primitives::AccountInfo;

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
}
