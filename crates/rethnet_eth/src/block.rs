// Part of this code was adapted from foundry and is distributed under their licenss:
// - https://github.com/foundry-rs/foundry/blob/01b16238ff87dc7ca8ee3f5f13e389888c2a2ee4/LICENSE-APACHE
// - https://github.com/foundry-rs/foundry/blob/01b16238ff87dc7ca8ee3f5f13e389888c2a2ee4/LICENSE-MIT
// For the original context see: https://github.com/foundry-rs/foundry/blob/01b16238ff87dc7ca8ee3f5f13e389888c2a2ee4/anvil/core/src/eth/block.rs

mod difficulty;
mod options;
mod reorg;

use std::sync::OnceLock;

use revm_primitives::{keccak256, ruint::aliases::U160, SpecId};
use rlp::Decodable;

use crate::{
    transaction::SignedTransaction,
    trie::{self, KECCAK_NULL_RLP},
    withdrawal::Withdrawal,
    Address, Bloom, Bytes, B256, B64, U256,
};

use self::difficulty::calculate_ethash_canonical_difficulty;
pub use self::{
    options::BlockOptions,
    reorg::{
        block_time, is_safe_block_number, largest_safe_block_number, safe_block_depth,
        IsSafeBlockNumberArgs, LargestSafeBlockNumberArgs,
    },
};

/// Ethereum block
#[derive(Clone, Debug, Eq)]
#[cfg_attr(
    feature = "fastrlp",
    derive(open_fastrlp::RlpEncodable, open_fastrlp::RlpDecodable)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Block {
    /// The block's header
    pub header: Header,
    /// The block's transactions
    pub transactions: Vec<SignedTransaction>,
    /// The block's ommers' headers
    pub ommers: Vec<Header>,
    /// The block's withdrawals
    pub withdrawals: Option<Vec<Withdrawal>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    /// The cached block hash
    hash: OnceLock<B256>,
}

impl Block {
    /// Constructs a new block from the provided partial header, transactions, and ommers.
    pub fn new(
        partial_header: PartialHeader,
        transactions: Vec<SignedTransaction>,
        ommers: Vec<Header>,
        withdrawals: Option<Vec<Withdrawal>>,
    ) -> Self {
        let ommers_hash = keccak256(&rlp::encode_list(&ommers)[..]);
        let transactions_root =
            trie::ordered_trie_root(transactions.iter().map(|r| rlp::encode(r).freeze()));

        let withdrawals_root = withdrawals.as_ref().map(|withdrawals| {
            trie::ordered_trie_root(withdrawals.iter().map(|r| rlp::encode(r).freeze()))
        });

        Self {
            header: Header::new(
                partial_header,
                ommers_hash,
                transactions_root,
                withdrawals_root,
            ),
            transactions,
            ommers,
            withdrawals,
            hash: OnceLock::new(),
        }
    }

    /// Retrieves the block's hash.
    pub fn hash(&self) -> &B256 {
        self.hash.get_or_init(|| self.header.hash())
    }
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header
            && self.transactions == other.transactions
            && self.ommers == other.ommers
            && self.withdrawals == other.withdrawals
    }
}

/// ethereum block header
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Header {
    /// The parent block's hash
    pub parent_hash: B256,
    /// The ommers' root hash
    pub ommers_hash: B256,
    /// The block's beneficiary address
    pub beneficiary: Address,
    /// The state's root hash
    pub state_root: B256,
    /// The transactions' root hash
    pub transactions_root: B256,
    /// The receipts' root hash
    pub receipts_root: B256,
    /// The logs' bloom
    pub logs_bloom: Bloom,
    /// The block's difficulty
    pub difficulty: U256,
    /// The block's number
    pub number: u64,
    /// The block's gas limit
    pub gas_limit: u64,
    /// The amount of gas used by the block
    pub gas_used: u64,
    /// The block's timestamp
    pub timestamp: u64,
    /// The block's extra data
    pub extra_data: Bytes,
    /// The block's mix hash
    pub mix_hash: B256,
    /// The block's nonce
    #[cfg_attr(feature = "serde", serde(with = "B64Def"))]
    pub nonce: B64,
    /// BaseFee was added by EIP-1559 and is ignored in legacy headers.
    pub base_fee_per_gas: Option<U256>,
    /// WithdrawalsHash was added by EIP-4895 and is ignored in legacy headers.
    pub withdrawals_root: Option<B256>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "B64")]
#[cfg(feature = "serde")]
struct B64Def(#[serde(getter = "B64::as_uint")] revm_primitives::ruint::aliases::U64);

#[cfg(feature = "serde")]
impl From<B64Def> for B64 {
    fn from(def: B64Def) -> Self {
        def.0.into()
    }
}

impl Header {
    /// Constructs a [`Header`] from the provided [`PartialHeader`], ommers' root hash, transactions' root hash, and withdrawals' root hash.
    pub fn new(
        partial_header: PartialHeader,
        ommers_hash: B256,
        transactions_root: B256,
        withdrawals_root: Option<B256>,
    ) -> Self {
        Self {
            parent_hash: partial_header.parent_hash,
            ommers_hash,
            beneficiary: partial_header.beneficiary,
            state_root: partial_header.state_root,
            transactions_root,
            receipts_root: partial_header.receipts_root,
            logs_bloom: partial_header.logs_bloom,
            difficulty: partial_header.difficulty,
            number: partial_header.number,
            gas_limit: partial_header.gas_limit,
            gas_used: partial_header.gas_used,
            timestamp: partial_header.timestamp,
            extra_data: partial_header.extra_data,
            mix_hash: partial_header.mix_hash,
            nonce: partial_header.nonce,
            base_fee_per_gas: partial_header.base_fee,
            withdrawals_root,
        }
    }

    /// Calculates the block's hash.
    pub fn hash(&self) -> B256 {
        let encoded = rlp::encode(self);
        keccak256(&encoded)
    }

    /// Returns the rlp length of the Header body, _not including_ trailing EIP155 fields or the
    /// rlp list header
    /// To get the length including the rlp list header, refer to the Encodable implementation.
    #[cfg(feature = "fastrlp")]
    pub(crate) fn header_payload_length(&self) -> usize {
        use open_fastrlp::Encodable;

        let mut length = 0;
        length += self.parent_hash.length();
        length += self.ommers_hash.length();
        length += self.beneficiary.length();
        length += self.state_root.length();
        length += self.transactions_root.length();
        length += self.receipts_root.length();
        length += self.logs_bloom.length();
        length += self.difficulty.length();
        length += self.number.length();
        length += self.gas_limit.length();
        length += self.gas_used.length();
        length += self.timestamp.length();
        length += self.extra_data.length();
        length += self.mix_hash.length();
        length += self.nonce.length();
        length += self
            .base_fee_per_gas
            .map(|fee| fee.length())
            .unwrap_or_default();
        length += self
            .withdrawals_root
            .map(|root| root.length())
            .unwrap_or_default();
        length
    }
}

impl rlp::Encodable for Header {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(if self.base_fee_per_gas.is_none() {
            15
        } else if self.withdrawals_root.is_none() {
            16
        } else {
            17
        });

        s.append(&self.parent_hash.as_bytes());
        s.append(&self.ommers_hash.as_bytes());
        s.append(&self.beneficiary.as_bytes());
        s.append(&self.state_root.as_bytes());
        s.append(&self.transactions_root.as_bytes());
        s.append(&self.receipts_root.as_bytes());
        s.append(&self.logs_bloom);
        s.append(&self.difficulty);
        s.append(&self.number);
        s.append(&self.gas_limit);
        s.append(&self.gas_used);
        s.append(&self.timestamp);
        s.append(&self.extra_data);
        s.append(&self.mix_hash.as_bytes());
        s.append(&self.nonce.to_le_bytes::<8>().as_ref());
        if let Some(ref base_fee) = self.base_fee_per_gas {
            s.append(base_fee);
        }
        if let Some(ref root) = self.withdrawals_root {
            s.append(&root.as_bytes());
        }
    }
}

impl rlp::Decodable for Header {
    fn decode(rlp: &rlp::Rlp<'_>) -> Result<Self, rlp::DecoderError> {
        let result = Header {
            parent_hash: B256::from(rlp.val_at::<U256>(0)?.to_be_bytes()),
            ommers_hash: B256::from(rlp.val_at::<U256>(1)?.to_be_bytes()),
            beneficiary: Address::from(rlp.val_at::<U160>(2)?.to_be_bytes()),
            state_root: B256::from(rlp.val_at::<U256>(3)?.to_be_bytes()),
            transactions_root: B256::from(rlp.val_at::<U256>(4)?.to_be_bytes()),
            receipts_root: B256::from(rlp.val_at::<U256>(5)?.to_be_bytes()),
            logs_bloom: rlp.val_at(6)?,
            difficulty: rlp.val_at(7)?,
            number: rlp.val_at(8)?,
            gas_limit: rlp.val_at(9)?,
            gas_used: rlp.val_at(10)?,
            timestamp: rlp.val_at(11)?,
            extra_data: rlp.val_at::<Vec<u8>>(12)?.into(),
            mix_hash: B256::from(rlp.val_at::<U256>(13)?.to_be_bytes()),
            nonce: B64::try_from_le_slice(&rlp.val_at::<Vec<u8>>(14)?)
                .ok_or(rlp::DecoderError::Custom("Invalid nonce byte length"))?,
            base_fee_per_gas: if let Ok(base_fee) = rlp.at(15) {
                Some(<U256 as Decodable>::decode(&base_fee)?)
            } else {
                None
            },
            withdrawals_root: if let Ok(root) = rlp.at(16) {
                Some(B256::from(
                    <U256 as Decodable>::decode(&root)?.to_be_bytes(),
                ))
            } else {
                None
            },
        };
        Ok(result)
    }
}

/// Partial header definition without ommers hash and transactions root
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartialHeader {
    /// The parent block's hash
    pub parent_hash: B256,
    /// The block's beneficiary address
    pub beneficiary: Address,
    /// The state's root hash
    pub state_root: B256,
    /// The receipts' root hash
    pub receipts_root: B256,
    /// The logs' bloom
    pub logs_bloom: Bloom,
    /// The block's difficulty
    pub difficulty: U256,
    /// The block's number
    pub number: u64,
    /// The block's gas limit
    pub gas_limit: u64,
    /// The amount of gas used by the block
    pub gas_used: u64,
    /// The block's timestamp
    pub timestamp: u64,
    /// The block's extra data
    pub extra_data: Bytes,
    /// The block's mix hash
    pub mix_hash: B256,
    /// The block's nonce
    pub nonce: B64,
    /// BaseFee was added by EIP-1559 and is ignored in legacy headers.
    pub base_fee: Option<U256>,
}

impl PartialHeader {
    /// Constructs a new instance based on the provided [`BlockOptions`] and parent [`Header`] for the given [`SpecId`].
    pub fn new(spec_id: SpecId, options: BlockOptions, parent: Option<&Header>) -> Self {
        let timestamp = options.timestamp.unwrap_or_default();
        let number = options.number.unwrap_or({
            if let Some(parent) = &parent {
                parent.number + 1
            } else {
                0
            }
        });

        let parent_hash = options.parent_hash.unwrap_or_else(|| {
            if let Some(parent) = parent {
                parent.hash()
            } else {
                B256::zero()
            }
        });

        Self {
            parent_hash,
            beneficiary: options.beneficiary.unwrap_or_default(),
            state_root: options.state_root.unwrap_or(KECCAK_NULL_RLP),
            receipts_root: options.receipts_root.unwrap_or(KECCAK_NULL_RLP),
            logs_bloom: options.logs_bloom.unwrap_or_default(),
            difficulty: options.difficulty.unwrap_or_else(|| {
                if spec_id >= SpecId::MERGE {
                    U256::ZERO
                } else if let Some(parent) = parent {
                    calculate_ethash_canonical_difficulty(spec_id, parent, number, timestamp)
                } else {
                    U256::from(1)
                }
            }),
            number,
            gas_limit: options.gas_limit.unwrap_or(1_000_000),
            gas_used: 0,
            timestamp,
            extra_data: options.extra_data.unwrap_or_default(),
            mix_hash: options.mix_hash.unwrap_or_default(),
            nonce: options.nonce.unwrap_or_default(),
            base_fee: options.base_fee.or_else(|| {
                if spec_id >= SpecId::LONDON {
                    Some(U256::from(7))
                } else {
                    None
                }
            }),
        }
    }
}

impl Default for PartialHeader {
    fn default() -> Self {
        const DEFAULT_GAS: u64 = 0xffffffffffffff;

        Self {
            parent_hash: B256::default(),
            beneficiary: Address::default(),
            state_root: B256::default(),
            receipts_root: KECCAK_NULL_RLP,
            logs_bloom: Bloom::default(),
            difficulty: U256::default(),
            number: u64::default(),
            gas_limit: DEFAULT_GAS,
            gas_used: u64::default(),
            timestamp: u64::default(),
            extra_data: Bytes::default(),
            mix_hash: B256::default(),
            nonce: B64::default(),
            base_fee: Option::default(),
        }
    }
}

impl From<Header> for PartialHeader {
    fn from(header: Header) -> PartialHeader {
        Self {
            parent_hash: header.parent_hash,
            beneficiary: header.beneficiary,
            state_root: header.state_root,
            receipts_root: header.receipts_root,
            logs_bloom: header.logs_bloom,
            difficulty: header.difficulty,
            number: header.number,
            gas_limit: header.gas_limit,
            gas_used: header.gas_used,
            timestamp: header.timestamp,
            extra_data: header.extra_data,
            mix_hash: header.mix_hash,
            nonce: header.nonce,
            base_fee: header.base_fee_per_gas,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use revm_primitives::ruint::aliases::U64;

    use super::*;

    #[test]
    fn header_rlp_roundtrip() {
        let mut header = Header {
            parent_hash: B256::default(),
            ommers_hash: B256::default(),
            beneficiary: Address::default(),
            state_root: B256::default(),
            transactions_root: B256::default(),
            receipts_root: B256::default(),
            logs_bloom: Bloom::default(),
            difficulty: U256::default(),
            number: 124,
            gas_limit: u64::default(),
            gas_used: 1337,
            timestamp: 0,
            extra_data: Bytes::default(),
            mix_hash: B256::default(),
            nonce: B64::from_limbs([99u64.to_be()]),
            base_fee_per_gas: None,
            withdrawals_root: None,
        };

        let encoded = rlp::encode(&header);
        let decoded: Header = rlp::decode(encoded.as_ref()).unwrap();
        assert_eq!(header, decoded);

        header.base_fee_per_gas = Some(U256::from(12345));

        let encoded = rlp::encode(&header);
        let decoded: Header = rlp::decode(encoded.as_ref()).unwrap();
        assert_eq!(header, decoded);
    }

    #[test]
    // Test vector from: https://eips.ethereum.org/EIPS/eip-2481
    fn test_encode_block_header() {
        let expected = hex::decode("f901f9a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000940000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008208ae820d0582115c8215b3821a0a827788a00000000000000000000000000000000000000000000000000000000000000000880000000000000000").unwrap();
        let header = Header {
            parent_hash: B256::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            ommers_hash: B256::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            beneficiary: Address::from_str("0000000000000000000000000000000000000000").unwrap(),
            state_root: B256::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            transactions_root: B256::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            receipts_root: B256::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            logs_bloom: Bloom::zero(),
            difficulty: U256::from(0x8aeu64),
            number: 0xd05,
            gas_limit: 0x115c,
            gas_used: 0x15b3,
            timestamp: 0x1a0a,
            extra_data: hex::decode("7788").unwrap().into(),
            mix_hash: B256::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            nonce: B64::from_limbs([0x0u64.to_be()]),
            base_fee_per_gas: None,
            withdrawals_root: None,
        };

        let encoded = rlp::encode(&header);
        assert_eq!(hex::encode(&encoded), hex::encode(expected));
    }

    #[test]
    // Test vector from: https://github.com/ethereum/tests/blob/f47bbef4da376a49c8fc3166f09ab8a6d182f765/BlockchainTests/ValidBlocks/bcEIP1559/baseFee.json#L15-L36
    fn test_eip1559_block_header_hash() {
        let expected_hash =
            B256::from_str("0x6a251c7c3c5dca7b42407a3752ff48f3bbca1fab7f9868371d9918daf1988d1f")
                .unwrap();
        let header = Header {
            parent_hash: B256::from_str(
                "0xe0a94a7a3c9617401586b1a27025d2d9671332d22d540e0af72b069170380f2a",
            )
            .unwrap(),
            ommers_hash: B256::from_str(
                "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
            )
            .unwrap(),
            beneficiary: Address::from_str("0xba5e000000000000000000000000000000000000").unwrap(),
            state_root: B256::from_str(
                "0xec3c94b18b8a1cff7d60f8d258ec723312932928626b4c9355eb4ab3568ec7f7",
            )
            .unwrap(),
            transactions_root: B256::from_str(
                "0x50f738580ed699f0469702c7ccc63ed2e51bc034be9479b7bff4e68dee84accf",
            )
            .unwrap(),
            receipts_root: B256::from_str(
                "0x29b0562f7140574dd0d50dee8a271b22e1a0a7b78fca58f7c60370d8317ba2a9",
            )
            .unwrap(),
            logs_bloom: Bloom::zero(),
            difficulty: U256::from(0x020000u64),
            number: 0x01,
            gas_limit: 0x016345785d8a0000,
            gas_used: 0x015534,
            timestamp: 0x079e,
            extra_data: hex::decode("42").unwrap().into(),
            mix_hash: B256::zero(),
            nonce: B64::from(U64::ZERO),
            base_fee_per_gas: Some(U256::from(0x036bu64)),
            withdrawals_root: None,
        };
        assert_eq!(header.hash(), expected_hash);
    }

    #[test]
    // Test vector from: https://eips.ethereum.org/EIPS/eip-2481
    fn test_decode_block_header() {
        let data = hex::decode("f901f9a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000940000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008208ae820d0582115c8215b3821a0a827788a00000000000000000000000000000000000000000000000000000000000000000880000000000000000").unwrap();
        let expected = Header {
            parent_hash: B256::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            ommers_hash: B256::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            beneficiary: Address::from_str("0000000000000000000000000000000000000000").unwrap(),
            state_root: B256::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            transactions_root: B256::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            receipts_root: B256::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            logs_bloom: Bloom::zero(),
            difficulty: U256::from(0x8aeu64),
            number: 0xd05,
            gas_limit: 0x115c,
            gas_used: 0x15b3,
            timestamp: 0x1a0a,
            extra_data: hex::decode("7788").unwrap().into(),
            mix_hash: B256::from_str(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            nonce: B64::from_limbs([0x0u64.to_be()]),
            base_fee_per_gas: None,
            withdrawals_root: None,
        };
        let header: Header = rlp::decode(&data).unwrap();
        assert_eq!(header, expected);
    }
}
