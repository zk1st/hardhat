use hashbrown::HashMap;
use rethnet_eth::{account::BasicAccount, state::state_root, Address, B256, U256};
use revm::{
    db::State,
    primitives::{Account, AccountInfo, Bytecode, KECCAK_EMPTY},
    DatabaseCommit,
};

use super::{RethnetStorage, StateDebug, StateError};

#[derive(Clone, Debug)]
struct RevertedLayers<Layer: Clone> {
    /// The parent layer's state root
    pub parent_state_root: B256,
    /// The reverted layers
    pub stack: Vec<Layer>,
}

/// A state consisting of layers.
#[derive(Clone, Debug)]
pub struct LayeredState<Layer: Clone> {
    stack: Vec<Layer>,
    /// The old parent layer state root and the reverted layers
    reverted_layers: Option<RevertedLayers<Layer>>,
    /// Snapshots
    snapshots: HashMap<B256, Vec<Layer>>, // naive implementation
}

impl<Layer: Clone> LayeredState<Layer> {
    /// Creates a [`LayeredState`] with the provided layer at the bottom.
    pub fn with_layer(layer: Layer) -> Self {
        Self {
            stack: vec![layer],
            reverted_layers: None,
            snapshots: HashMap::new(),
        }
    }

    /// Returns the index of the top layer.
    pub fn last_layer_id(&self) -> usize {
        self.stack.len() - 1
    }

    /// Returns an immutable reference to the top layer.
    pub fn last_layer(&self) -> &Layer {
        // The `LayeredState` always has at least one layer
        self.stack.last().unwrap()
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

    /// Returns a mutable iterator over the object's layers.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Layer> {
        self.stack.iter_mut().rev()
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
            reverted_layers: None,
            snapshots: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RethnetAccount {
    info: AccountInfo,
    storage: RethnetStorage,
}

impl RethnetAccount {
    pub fn new(info: AccountInfo) -> Self {
        Self {
            info,
            storage: RethnetStorage::default(),
        }
    }
}

/// A layer with information needed for [`Rethnet`].
#[derive(Clone, Debug, Default)]
pub struct RethnetLayer {
    /// Address -> Account
    accounts: HashMap<Address, RethnetAccount>,
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
            .map(|(address, account_info)| (address, RethnetAccount::new(account_info)))
            .collect();

        Self {
            accounts: genesis_accounts,
            ..Default::default()
        }
    }

    pub fn state_root(&mut self) -> B256 {
        if let Some(state_root) = self.state_root {
            state_root
        } else {
            let state = self
                .accounts
                .iter_mut()
                .map(|(address, RethnetAccount { info, storage })| {
                    let storage_root = storage.storage_root();
                    let account = BasicAccount {
                        nonce: U256::from(info.nonce),
                        balance: info.balance,
                        storage_root,
                        code_hash: info.code_hash,
                    };

                    (*address, account)
                })
                .collect();

            let state_root = state_root(&state);
            self.state_root = Some(state_root);
            state_root
        }
    }

    /// Returns whether the layer has a state root.
    pub fn has_state_root(&self) -> bool {
        self.state_root.is_some()
    }

    /// Insert the provided `AccountInfo` at the specified `address`.
    pub fn insert_account(&mut self, address: Address, mut account: RethnetAccount) {
        if let Some(code) = account.info.code.take() {
            if !code.is_empty() {
                account.info.code_hash = code.hash();
                self.contracts.insert(code.hash(), code);
            }
        }

        if account.info.code_hash.is_zero() {
            account.info.code_hash = KECCAK_EMPTY;
        }

        self.accounts.insert(address, account);
    }
}

impl LayeredState<RethnetLayer> {
    /// Removes the [`RethnetAccount`] corresponding to the specified address.
    fn remove_account(&mut self, address: &Address) -> Option<RethnetAccount> {
        let account = self.last_layer_mut().accounts.remove(address);

        if let Some(account) = &account {
            debug_assert!(account.info.code.is_none());

            self.last_layer_mut()
                .contracts
                .remove(&account.info.code_hash);
        }

        account
    }
}

impl State for LayeredState<RethnetLayer> {
    type Error = StateError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let account = self.last_layer().accounts.get(&address);

        // TODO: Move this out of LayeredState when forking
        Ok(account.map_or(
            Some(AccountInfo {
                balance: U256::ZERO,
                nonce: 0,
                code_hash: KECCAK_EMPTY,
                code: None,
            }),
            |account| Some(account.info.clone()),
        ))
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        if code_hash == KECCAK_EMPTY {
            return Ok(Bytecode::new());
        }

        self.iter()
            .find_map(|layer| layer.contracts.get(&code_hash).cloned())
            .ok_or(StateError::InvalidCodeHash(code_hash))
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        Ok(self
            .last_layer()
            .accounts
            .get(&address)
            .and_then(|account| account.storage.get(&index))
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
                self.last_layer_mut()
                    .accounts
                    .entry(address)
                    .and_modify(|old_account| {
                        old_account.info = account.info.clone();

                        if account.storage_cleared {
                            old_account.storage = RethnetStorage::default();
                        }

                        account.storage.iter().for_each(|(index, value)| {
                            let value = value.present_value();
                            if value == U256::ZERO {
                                old_account.storage.remove(index);
                            } else {
                                old_account.storage.insert(*index, value);
                            }
                        });
                    })
                    .or_insert_with(|| RethnetAccount {
                        info: account.info,
                        storage: RethnetStorage::new(
                            account
                                .storage
                                .into_iter()
                                .map(|(index, value)| (index, value.present_value))
                                .filter(|(_index, value)| *value == U256::ZERO)
                                .collect(),
                        ),
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
            .accounts
            .get_mut(address)
            .map(|account| account.storage.storage_root()))
    }

    fn insert_account(
        &mut self,
        address: Address,
        account_info: AccountInfo,
    ) -> Result<(), Self::Error> {
        self.last_layer_mut()
            .insert_account(address, RethnetAccount::new(account_info));

        Ok(())
    }

    fn make_snapshot(&mut self) -> B256 {
        let state_root = self.state_root().unwrap();
        let mut snapshot = self.stack.clone();
        if let Some(layer) = snapshot.last_mut() {
            layer.state_root.replace(state_root);
        }

        // Currently overwrites old snapshots
        self.snapshots.insert(state_root, snapshot);

        state_root
    }

    fn modify_account(
        &mut self,
        address: Address,
        modifier: Box<dyn Fn(&mut U256, &mut u64, &mut Option<Bytecode>) + Send>,
    ) -> Result<(), Self::Error> {
        let account = self
            .last_layer_mut()
            .accounts
            .entry(address)
            .or_insert_with(|| {
                RethnetAccount::new(AccountInfo {
                    balance: U256::ZERO,
                    nonce: 0,
                    code_hash: KECCAK_EMPTY,
                    code: None,
                })
            });

        let old_code_hash = account.info.code_hash;

        modifier(
            &mut account.info.balance,
            &mut account.info.nonce,
            &mut account.info.code,
        );

        if let Some(code) = account.info.code.take() {
            let new_code_hash = code.hash();

            if old_code_hash != new_code_hash {
                account.info.code_hash = new_code_hash;

                let last_layer = self.last_layer_mut();

                // The old contract should now return empty bytecode
                last_layer.contracts.insert(old_code_hash, Bytecode::new());

                last_layer.contracts.insert(new_code_hash, code);
            }
        }

        Ok(())
    }

    fn remove_account(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(self
            .last_layer_mut()
            .accounts
            .remove(&address)
            .map(|account| account.info))
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
            .accounts
            .entry(address)
            .and_modify(|account| {
                account.storage.insert(index, value);
            })
            .or_insert_with(|| {
                let mut slots = HashMap::new();
                slots.insert(index, value);

                RethnetAccount {
                    info: AccountInfo {
                        balance: U256::ZERO,
                        nonce: 0,
                        code_hash: KECCAK_EMPTY,
                        code: None,
                    },
                    storage: RethnetStorage::new(slots),
                }
            });

        Ok(())
    }

    fn set_state_root(&mut self, state_root: &B256) -> Result<(), Self::Error> {
        if let Some(snapshot) = self.snapshots.get(state_root) {
            // Retain all layers except the first
            self.reverted_layers = Some(RevertedLayers {
                parent_state_root: self.stack.first_mut().unwrap().state_root(),
                stack: self.stack.split_off(1),
            });
            self.stack = snapshot.clone();

            return Ok(());
        }

        // Check whether the state root is contained in the previously reverted layers
        let reinstated_layers = self.reverted_layers.take().and_then(|mut reverted_layers| {
            let layer_id =
                reverted_layers
                    .stack
                    .iter_mut()
                    .enumerate()
                    .find_map(|(layer_id, layer)| {
                        if layer.state_root() == *state_root {
                            Some(layer_id)
                        } else {
                            None
                        }
                    });

            if let Some(layer_id) = layer_id {
                reverted_layers.stack.truncate(layer_id + 1);

                Some(reverted_layers)
            } else {
                None
            }
        });

        let state_root = reinstated_layers
            .as_ref()
            .map_or(state_root, |reinstated_layers| {
                &reinstated_layers.parent_state_root
            });

        let layer_id = self
            .stack
            .iter_mut()
            .enumerate()
            .find_map(|(layer_id, layer)| {
                if layer.state_root() == *state_root {
                    Some(layer_id)
                } else {
                    None
                }
            });

        if let Some(layer_id) = layer_id {
            let reverted_layers = self.stack.split_off(layer_id + 1);
            let parent_state_root = self.stack.last_mut().unwrap().state_root();

            if let Some(mut reinstated_layers) = reinstated_layers {
                self.stack.append(&mut reinstated_layers.stack);
            }

            self.add_layer_default();

            self.reverted_layers = if reverted_layers.is_empty() {
                None
            } else {
                Some(RevertedLayers {
                    parent_state_root,
                    stack: reverted_layers,
                })
            };

            Ok(())
        } else {
            Err(StateError::InvalidStateRoot(*state_root))
        }
    }

    fn state_root(&mut self) -> Result<B256, Self::Error> {
        Ok(self.last_layer_mut().state_root())
    }

    fn checkpoint(&mut self) -> Result<(), Self::Error> {
        self.add_layer(self.last_layer().clone());

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
