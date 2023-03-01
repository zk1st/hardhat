use hashbrown::HashMap;
use rethnet_eth::{
    account::BasicAccount, state::state_root, trie::KECCAK_NULL_RLP, Address, B256, U256,
};
use revm::{
    db::State,
    primitives::{Account, AccountInfo, Bytecode, KECCAK_EMPTY},
    DatabaseCommit,
};

use super::{storage::RethnetStorage, StateDebug, StateError};

/// A state consisting of layers.
#[derive(Clone, Debug)]
pub struct LayeredState<Layer: Clone> {
    stack: Vec<Layer>,
    /// Snapshots
    snapshots: HashMap<B256, Vec<Layer>>, // naive implementation
}

impl<Layer: Clone> LayeredState<Layer> {
    /// Creates a [`LayeredState`] with the provided layer at the bottom.
    pub fn with_layer(layer: Layer) -> Self {
        Self {
            stack: vec![layer],
            snapshots: HashMap::new(),
        }
    }

    /// Returns the index of the top layer.
    pub fn last_layer_id(&self) -> usize {
        self.stack.len() - 1
    }

    /// Returns a mutable reference to the top layer.
    pub fn last_layer_mut(&mut self) -> &mut Layer {
        // The `LayeredState` always has at least one layer
        self.stack.last_mut().unwrap()
    }

    /// Adds the provided layer to the top, returning its index and a
    /// mutable reference to the layer.
    pub fn add_layer(&mut self, layer: Layer) -> (usize, &mut Layer) {
        let layer_id = self.stack.len();
        self.stack.push(layer);
        (layer_id, self.stack.last_mut().unwrap())
    }

    /// Reverts to the layer with specified `layer_id`, removing all
    /// layers above it.
    pub fn revert_to_layer(&mut self, layer_id: usize) {
        assert!(layer_id < self.stack.len(), "Invalid layer id.");
        self.stack.truncate(layer_id + 1);
    }

    /// Returns an iterator over the object's layers.
    pub fn iter(&self) -> impl Iterator<Item = &Layer> {
        self.stack.iter().rev()
    }
}

impl<Layer: Clone + Default> LayeredState<Layer> {
    /// Adds a default layer to the top, returning its index and a
    /// mutable reference to the layer.
    pub fn add_layer_default(&mut self) -> (usize, &mut Layer) {
        self.add_layer(Layer::default())
    }
}

impl<Layer: Clone + Default> Default for LayeredState<Layer> {
    fn default() -> Self {
        Self {
            stack: vec![Layer::default()],
            snapshots: HashMap::new(),
        }
    }
}

/// A layer with information needed for [`Rethnet`].
#[derive(Clone, Debug, Default)]
pub struct RethnetLayer {
    /// Address -> AccountInfo
    account_infos: HashMap<Address, AccountInfo>,
    /// Address -> Storage
    storage: HashMap<Address, RethnetStorage>,
    /// Code hash -> Address
    contracts: HashMap<B256, Bytecode>,
    /// Cached state root
    state_root: Option<B256>,
}

impl RethnetLayer {
    /// Creates a `RethnetLayer` with the provided genesis accounts.
    pub fn with_genesis_accounts(genesis_accounts: HashMap<Address, AccountInfo>) -> Self {
        let genesis_accounts = genesis_accounts
            .into_iter()
            .map(|(address, account_info)| (address, account_info))
            .collect();

        Self {
            account_infos: genesis_accounts,
            ..Default::default()
        }
    }

    /// Returns whether the layer has a state root.
    pub fn has_state_root(&self) -> bool {
        self.state_root.is_some()
    }

    /// Insert the provided `AccountInfo` at the specified `address`.
    pub fn insert_account(&mut self, address: Address, mut account_info: AccountInfo) {
        if let Some(code) = account_info.code.take() {
            if !code.is_empty() {
                account_info.code_hash = code.hash();
                self.contracts.insert(code.hash(), code);
            }
        }

        if account_info.code_hash.is_zero() {
            account_info.code_hash = KECCAK_EMPTY;
        }

        let new_code_hash = account_info.code_hash;

        if let Some(old_account) = self.account_infos.insert(address, account_info) {
            if old_account.code_hash != new_code_hash {
                self.contracts.remove(&old_account.code_hash);
            }
        }
    }
}

impl LayeredState<RethnetLayer> {
    /// Removes the [`AccountInfo`] corresponding to the specified address.
    fn remove_account(&mut self, address: &Address) -> Option<AccountInfo> {
        let account_info = self.last_layer_mut().account_infos.remove(address);

        if let Some(account_info) = &account_info {
            debug_assert!(account_info.code.is_none());

            self.last_layer_mut()
                .contracts
                .remove(&account_info.code_hash);
        }

        self.last_layer_mut().storage.remove(address);

        account_info
    }
}

impl State for LayeredState<RethnetLayer> {
    type Error = StateError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let account = self.last_layer_mut().account_infos.get(&address).cloned();

        // TODO: Move this out of LayeredState when forking
        Ok(account.or(Some(AccountInfo {
            balance: U256::ZERO,
            nonce: 0,
            code_hash: KECCAK_EMPTY,
            code: None,
        })))
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        if code_hash == KECCAK_EMPTY {
            return Ok(Bytecode::new());
        }

        self.last_layer_mut()
            .contracts
            .get(&code_hash)
            .cloned()
            .ok_or(StateError::InvalidCodeHash(code_hash))
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        Ok(self
            .last_layer_mut()
            .storage
            .get(&address)
            .and_then(|storage| storage.get(&index))
            .cloned()
            .unwrap_or(U256::ZERO))
    }
}

impl DatabaseCommit for LayeredState<RethnetLayer> {
    fn commit(&mut self, changes: HashMap<Address, Account>) {
        changes.into_iter().for_each(|(address, account)| {
            if account.is_empty() || account.is_destroyed {
                self.remove_account(&address);
            } else {
                self.last_layer_mut().insert_account(address, account.info);

                let storage = self.last_layer_mut().storage.entry(address).or_default();

                if account.storage_cleared {
                    *storage = RethnetStorage::default();
                }

                account.storage.into_iter().for_each(|(index, value)| {
                    let value = value.present_value();
                    if value == U256::ZERO {
                        storage.remove(&index);
                    } else {
                        storage.insert(index, value);
                    }
                });
            }
        });
    }
}

impl StateDebug for LayeredState<RethnetLayer> {
    type Error = StateError;

    fn account_storage_root(&mut self, address: &Address) -> Result<Option<B256>, Self::Error> {
        Ok(self
            .last_layer_mut()
            .storage
            .get_mut(address)
            .map(RethnetStorage::storage_root))
    }

    fn insert_account(
        &mut self,
        address: Address,
        account_info: AccountInfo,
    ) -> Result<(), Self::Error> {
        self.last_layer_mut().insert_account(address, account_info);

        Ok(())
    }

    fn make_snapshot(&mut self) -> (B256, bool) {
        let state_root = self.state_root().unwrap();
        let mut exists = true;
        self.snapshots.entry(state_root).or_insert_with(|| {
            exists = false;

            let mut snapshot = self.stack.clone();
            if let Some(layer) = snapshot.last_mut() {
                layer.state_root.replace(state_root);
            }
            snapshot
        });

        (state_root, exists)
    }

    fn modify_account(
        &mut self,
        address: Address,
        modifier: Box<dyn Fn(&mut U256, &mut u64, &mut Option<Bytecode>) + Send>,
    ) -> Result<(), Self::Error> {
        let account_info = self
            .last_layer_mut()
            .account_infos
            .entry(address)
            .or_default();

        let old_code_hash = account_info.code_hash;

        modifier(
            &mut account_info.balance,
            &mut account_info.nonce,
            &mut account_info.code,
        );

        if let Some(code) = account_info.code.take() {
            let new_code_hash = code.hash();

            if old_code_hash != new_code_hash {
                account_info.code_hash = new_code_hash;

                let last_layer = self.last_layer_mut();

                // The old contract should now return empty bytecode
                last_layer.contracts.insert(old_code_hash, Bytecode::new());

                last_layer.contracts.insert(new_code_hash, code);
            }
        }

        Ok(())
    }

    fn remove_account(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(self.remove_account(&address))
    }

    fn remove_snapshot(&mut self, state_root: &B256) -> bool {
        self.snapshots.remove(state_root).is_some()
    }

    fn set_account_storage_slot(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Result<(), Self::Error> {
        self.last_layer_mut()
            .storage
            .entry(address)
            .or_default()
            .insert(index, value);

        Ok(())
    }

    fn set_state_root(&mut self, state_root: &B256) -> Result<(), Self::Error> {
        // Ensure the last layer has a state root
        if !self.last_layer_mut().has_state_root() {
            let state_root = self.state_root()?;
            self.last_layer_mut().state_root.replace(state_root);
        }

        if let Some(snapshot) = self.snapshots.get(state_root) {
            self.stack = snapshot.clone();

            return Ok(());
        }

        Err(StateError::InvalidStateRoot(*state_root))
    }

    fn state_root(&mut self) -> Result<B256, Self::Error> {
        let storage_roots: HashMap<Address, B256> = self
            .last_layer_mut()
            .storage
            .iter_mut()
            .map(|(address, storage)| (*address, storage.storage_root()))
            .collect();

        let state = self
            .last_layer_mut()
            .account_infos
            .iter()
            .map(|(address, account_info)| {
                let storage_root = storage_roots
                    .get(address)
                    .cloned()
                    .unwrap_or(KECCAK_NULL_RLP);

                let account = BasicAccount {
                    nonce: U256::from(account_info.nonce),
                    balance: account_info.balance,
                    storage_root,
                    code_hash: account_info.code_hash,
                };

                (*address, account)
            })
            .collect();

        Ok(state_root(&state))
    }

    fn checkpoint(&mut self) -> Result<(), Self::Error> {
        let state_root = self.state_root()?;

        let last_layer = self.last_layer_mut().clone();
        self.last_layer_mut().state_root.replace(state_root);
        self.add_layer(last_layer);

        Ok(())
    }

    fn revert(&mut self) -> Result<(), Self::Error> {
        let last_layer_id = self.last_layer_id();
        if last_layer_id > 0 {
            self.revert_to_layer(last_layer_id - 1);
            Ok(())
        } else {
            Err(StateError::CannotRevert)
        }
    }
}
