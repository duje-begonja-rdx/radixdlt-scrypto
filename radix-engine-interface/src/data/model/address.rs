use crate::abi::*;
use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::data::ScryptoCustomValueKind;
use crate::*;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt::Debug;
use transaction_data::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Address {
    Package(PackageAddress),
    Component(ComponentAddress),
    Resource(ResourceAddress),
}

impl Address {
    pub fn encode_body_common<X: CustomValueKind, E: Encoder<X>>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        match self {
            Address::Package(v) => {
                encoder.write_byte(0)?;
                encoder.write_slice(&v.to_vec())?;
            }
            Address::Component(v) => {
                encoder.write_byte(1)?;
                encoder.write_slice(&v.to_vec())?;
            }
            Address::Resource(v) => {
                encoder.write_byte(2)?;
                encoder.write_slice(&v.to_vec())?;
            }
        }
        Ok(())
    }

    pub fn decode_body_common<X: CustomValueKind, D: Decoder<X>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        match decoder.read_byte()? {
            0 => Ok(Self::Package(
                PackageAddress::try_from(decoder.read_slice(27)?)
                    .map_err(|_| DecodeError::InvalidCustomValue)?,
            )),
            1 => Ok(Self::Component(
                ComponentAddress::try_from(decoder.read_slice(27)?)
                    .map_err(|_| DecodeError::InvalidCustomValue)?,
            )),
            2 => Ok(Self::Resource(
                ResourceAddress::try_from(decoder.read_slice(27)?)
                    .map_err(|_| DecodeError::InvalidCustomValue)?,
            )),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl Into<ComponentAddress> for Address {
    fn into(self) -> ComponentAddress {
        match self {
            Address::Component(component_address) => component_address,
            _ => panic!("Not a component address"),
        }
    }
}

impl Into<PackageAddress> for Address {
    fn into(self) -> PackageAddress {
        match self {
            Address::Package(package_address) => package_address,
            _ => panic!("Not a package address"),
        }
    }
}

impl Into<ResourceAddress> for Address {
    fn into(self) -> ResourceAddress {
        match self {
            Address::Resource(resource_address) => resource_address,
            _ => panic!("Not a resource address"),
        }
    }
}

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Address {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Address)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Address {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.encode_body_common(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Address {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        Self::decode_body_common(decoder)
    }
}

impl scrypto_abi::LegacyDescribe for Address {
    fn describe() -> scrypto_abi::Type {
        Type::Address
    }
}

//==================
// binary (manifest)
//==================

impl Categorize<ManifestCustomValueKind> for Address {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::Address)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E> for Address {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.encode_body_common(encoder)
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D> for Address {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        Self::decode_body_common(decoder)
    }
}
