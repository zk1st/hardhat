use revm_primitives::Address;
use rlp::{DecoderError, Rlp, RlpStream};
use ruint::aliases::U160;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransactionKind {
    Call(Address),
    Create,
}

impl TransactionKind {
    /// If this transaction is a call this returns the address of the callee
    pub fn as_call(&self) -> Option<&Address> {
        match self {
            TransactionKind::Call(to) => Some(to),
            TransactionKind::Create => None,
        }
    }
}

impl rlp::Encodable for TransactionKind {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            TransactionKind::Call(address) => {
                s.encoder().encode_value(&address[..]);
            }
            TransactionKind::Create => s.encoder().encode_value(&[]),
        }
    }
}

impl rlp::Decodable for TransactionKind {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        if rlp.is_empty() {
            if rlp.is_data() {
                Ok(TransactionKind::Create)
            } else {
                Err(DecoderError::RlpExpectedToBeData)
            }
        } else {
            Ok(TransactionKind::Call({
                let address = rlp.as_val::<U160>()?.to_be_bytes();
                Address::from(address)
            }))
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TransactionKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(if let Some(to) = Option::deserialize(deserializer)? {
            Self::Call(to)
        } else {
            Self::Create
        })
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for TransactionKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_call().serialize(serializer)
    }
}
