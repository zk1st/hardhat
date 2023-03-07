use hashbrown::HashMap;
use memory_db::{HashKey, MemoryDB};
use reference_trie::RefSecTrieDBMut;
use trie_db::TrieMut;

use crate::{account::BasicAccount, trie::sec_trie_root, Address, B256, U256};

/// State mapping of addresses to accounts.
pub type State = HashMap<Address, BasicAccount>;

/// Account storage mapping of indices to values.
pub type Storage = HashMap<U256, U256>;

/// Calculates the state root hash of the provided state.
pub fn state_root(state: &State) -> B256 {
    sec_trie_root(state.iter().map(|(address, account)| {
        let account = rlp::encode(account);
        (address, account)
    }))
}

/// Calculates the state root hash of the provided state.
pub fn state_root2(state: &State) -> B256 {
    let mut db = MemoryDB::<_, HashKey<_>, _>::default();
    let mut root = Default::default();

    {
        let mut trie = RefSecTrieDBMut::new(&mut db, &mut root);
        state
            .iter()
            .map(|(address, account)| {
                let account = rlp::encode(account);
                (address, account)
            })
            .for_each(|(address, value)| {
                trie.insert(address.as_ref(), value.as_ref()).unwrap();
            });
    }

    B256::from(root)
}

/// Calculates the storage root hash of the provided storage.
pub fn storage_root(storage: &Storage) -> B256 {
    sec_trie_root(storage.iter().map(|(index, value)| {
        let value = rlp::encode(value);
        (index.to_be_bytes::<32>(), value)
    }))
}

/// Calculates the storage root hash of the provided storage.
pub fn storage_root2(storage: &Storage) -> B256 {
    let mut db = MemoryDB::<_, HashKey<_>, _>::default();
    let mut root = Default::default();

    {
        let mut trie = RefSecTrieDBMut::new(&mut db, &mut root);
        storage
            .iter()
            .map(|(index, value)| {
                let value = rlp::encode(value);
                (index.to_be_bytes::<32>(), value)
            })
            .for_each(|(index, value)| {
                trie.insert(&index, value.as_ref()).unwrap();
            });
    }

    B256::from(root)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::trie::KECCAK_NULL_RLP;

    use super::*;

    #[test]
    fn empty_state_root() {
        let state = State::default();

        assert_eq!(state_root(&state), KECCAK_NULL_RLP);
    }

    #[test]
    fn empty_storage_root() {
        let storage = Storage::default();

        assert_eq!(storage_root(&storage), KECCAK_NULL_RLP);
    }

    #[test]
    fn precompiles_state_root() {
        let mut state = State::default();

        for idx in 1..=8u8 {
            let mut address = Address::zero();
            address.0[19] = idx;
            state.insert(address, BasicAccount::default());
        }

        const EXPECTED: &str = "0x5766c887a7240e4d1c035ccd3830a2f6a0c03d213a9f0b9b27c774916a4abcce";
        assert_eq!(state_root(&state), B256::from_str(EXPECTED).unwrap())
    }
}
