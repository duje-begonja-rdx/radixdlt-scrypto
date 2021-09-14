use sbor::{describe::Type, *};

use crate::constants::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::vec::Vec;
use crate::types::*;

/// Resource bucket id.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum BID {
    Transient(u32),

    Persisted(H256, u32),
}

/// Represents an error when parsing BID.
#[derive(Debug, Clone)]
pub enum ParseBIDError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

impl BID {
    pub fn is_transient(&self) -> bool {
        matches!(self, Self::Transient(_))
    }

    pub fn is_persisted(&self) -> bool {
        matches!(self, Self::Persisted(..))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Self::Transient(id) => combine2(0, &id.to_le_bytes()),
            Self::Persisted(hash, id) => combine3(1, hash.as_ref(), &id.to_le_bytes()),
        }
    }
}

impl FromStr for BID {
    type Err = ParseBIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseBIDError::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&[u8]> for BID {
    type Error = ParseBIDError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match (slice.get(0), slice.len()) {
            (Some(0), 5) => Ok(BID::Transient(u32::from_le_bytes(copy_u8_array(
                &slice[1..],
            )))),
            (Some(1), 37) => Ok(BID::Persisted(
                H256(copy_u8_array(&slice[1..33])),
                u32::from_le_bytes(copy_u8_array(&slice[33..])),
            )),
            (_, len) => Err(ParseBIDError::InvalidLength(len)),
        }
    }
}

impl fmt::Debug for BID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Display for BID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl Encode for BID {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }

    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_BID
    }
}

impl Decode for BID {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_BID))
    }

    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_BID
    }
}

impl Describe for BID {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_BID.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::string::ToString;

    #[test]
    fn test_from_to_string() {
        let s = "01f4cb57e4c4cd9d6564823eee427779d022d4f5f601791484a97837e6ffcf4cba01000000";
        let a = BID::from_str(s).unwrap();
        assert_eq!(a.to_string(), s);
    }
}