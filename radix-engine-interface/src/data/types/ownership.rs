use crate::abi::*;
use crate::api::types::*;
use crate::data::ScryptoCustomTypeId;
use crate::scrypto;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use utils::copy_u8_array;

// TODO: it's still up to debate whether this should be an enum OR dedicated types for each variant.
#[scrypto(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Own {
    Bucket(BucketId),
    Proof(ProofId),
    Vault(VaultId),
}

impl Own {
    pub fn vault_id(&self) -> VaultId {
        match self {
            Own::Vault(v) => v.clone(),
            _ => panic!("Not a vault ownership"),
        }
    }
    pub fn bucket_id(&self) -> BucketId {
        match self {
            Own::Bucket(v) => v.clone(),
            _ => panic!("Not a bucket ownership"),
        }
    }
    pub fn proof_id(&self) -> ProofId {
        match self {
            Own::Proof(v) => v.clone(),
            _ => panic!("Not a proof ownership"),
        }
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseOwnError {
    InvalidLength(usize),
    UnknownVariant(u8),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseOwnError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseOwnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TypeId<ScryptoCustomTypeId> for Own {
    #[inline]
    fn type_id() -> SborTypeId<ScryptoCustomTypeId> {
        SborTypeId::Custom(ScryptoCustomTypeId::Own)
    }
}

impl<E: Encoder<ScryptoCustomTypeId>> Encode<ScryptoCustomTypeId, E> for Own {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(Self::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Own::Bucket(v) => {
                encoder.write_byte(0)?;
                encoder.write_slice(&v.to_le_bytes())?;
            }
            Own::Proof(v) => {
                encoder.write_byte(1)?;
                encoder.write_slice(&v.to_le_bytes())?;
            }
            Own::Vault(v) => {
                encoder.write_byte(2)?;
                encoder.write_slice(v)?;
            }
        }
        Ok(())
    }
}

impl<D: Decoder<ScryptoCustomTypeId>> Decode<ScryptoCustomTypeId, D> for Own {
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<ScryptoCustomTypeId>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        match decoder.read_byte()? {
            0 => Ok(Self::Bucket(u32::from_le_bytes(copy_u8_array(
                decoder.read_slice(4)?,
            )))),
            1 => Ok(Self::Proof(u32::from_le_bytes(copy_u8_array(
                decoder.read_slice(4)?,
            )))),
            2 => Ok(Self::Vault(copy_u8_array(decoder.read_slice(36)?))),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl scrypto_abi::Describe for Own {
    fn describe() -> scrypto_abi::Type {
        Type::Own
    }
}