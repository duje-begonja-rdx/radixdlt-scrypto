use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::address::*;
use crate::constants::*;
use crate::crypto::*;
use crate::data::ScryptoCustomTypeId;
use crate::data::ScryptoSborTypeId;
use crate::model::*;
use utils::ContextualDisplay;

/// Identifier for a non-fungible unit.
#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct NonFungibleAddress {
    pub resource_address: ResourceAddress,
    pub non_fungible_id: NonFungibleId,
}

//=======
// error
//=======

/// Represents an error when parsing non-fungible address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleAddressError {
    InvalidLength(usize),
    InvalidResourceAddress(AddressError),
    InvalidNonFungibleId(ParseNonFungibleIdError),
    RequiresTwoParts,
}

impl From<AddressError> for ParseNonFungibleAddressError {
    fn from(err: AddressError) -> Self {
        Self::InvalidResourceAddress(err)
    }
}

impl From<ParseNonFungibleIdError> for ParseNonFungibleAddressError {
    fn from(err: ParseNonFungibleIdError) -> Self {
        Self::InvalidNonFungibleId(err)
    }
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseNonFungibleAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseNonFungibleAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TypeId<ScryptoCustomTypeId> for NonFungibleAddress {
    #[inline]
    fn type_id() -> SborTypeId<ScryptoCustomTypeId> {
        SborTypeId::Custom(ScryptoCustomTypeId::NonFungibleAddress)
    }
}

impl<E: Encoder<ScryptoCustomTypeId>> Encode<ScryptoCustomTypeId, E> for NonFungibleAddress {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(Self::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.resource_address.encode_body(encoder)?;
        self.non_fungible_id.encode_body(encoder)?;
        Ok(())
    }
}

impl<D: Decoder<ScryptoCustomTypeId>> Decode<ScryptoCustomTypeId, D> for NonFungibleAddress {
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<ScryptoCustomTypeId>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        Ok(Self {
            resource_address: ResourceAddress::decode_body_with_type_id(
                decoder,
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::ResourceAddress),
            )?,
            non_fungible_id: NonFungibleId::decode_body_with_type_id(
                decoder,
                ScryptoSborTypeId::Custom(ScryptoCustomTypeId::NonFungibleId),
            )?,
        })
    }
}

impl scrypto_abi::LegacyDescribe for NonFungibleAddress {
    fn describe() -> scrypto_abi::Type {
        Type::NonFungibleAddress
    }
}

//========
// impl
//========

impl NonFungibleAddress {
    pub const fn new(resource_address: ResourceAddress, non_fungible_id: NonFungibleId) -> Self {
        Self {
            resource_address,
            non_fungible_id,
        }
    }

    /// Returns the resource address.
    pub fn resource_address(&self) -> ResourceAddress {
        self.resource_address
    }

    /// Returns the non-fungible id.
    pub fn non_fungible_id(&self) -> &NonFungibleId {
        &self.non_fungible_id
    }

    /// Returns canonical representation of this NonFungibleAddress.
    pub fn to_canonical_string(&self, bech32_encoder: &Bech32Encoder) -> String {
        format!(
            "{}:{}",
            bech32_encoder.encode_resource_address_to_string(&self.resource_address),
            self.non_fungible_id.to_simple_string()
        )
    }

    /// Converts canonical representation to NonFungibleAddress.
    ///
    /// This is composed of `resource_address:id_simple_representation`
    pub fn try_from_canonical_string(
        bech32_decoder: &Bech32Decoder,
        id_type: NonFungibleIdTypeId,
        s: &str,
    ) -> Result<Self, ParseNonFungibleAddressError> {
        let v = s
            .splitn(2, ':')
            .filter(|&s| !s.is_empty())
            .collect::<Vec<&str>>();
        if v.len() != 2 {
            return Err(ParseNonFungibleAddressError::RequiresTwoParts);
        }
        let resource_address = bech32_decoder.validate_and_decode_resource_address(v[0])?;
        let non_fungible_id = NonFungibleId::try_from_simple_string(id_type, v[1])?;
        Ok(NonFungibleAddress::new(resource_address, non_fungible_id))
    }

    /// Returns canonical representation of this NonFungibleAddress.
    pub fn to_canonical_combined_string(&self, bech32_encoder: &Bech32Encoder) -> String {
        format!(
            "{}:{}",
            bech32_encoder.encode_resource_address_to_string(&self.resource_address),
            self.non_fungible_id.to_combined_simple_string()
        )
    }

    /// Converts combined canonical representation to NonFungibleAddress.
    ///
    /// This is composed of `resource_address:IdType#id_simple_representation`
    ///
    /// Prefer the canonical string where the id type can be looked up.
    pub fn try_from_canonical_combined_string(
        bech32_decoder: &Bech32Decoder,
        s: &str,
    ) -> Result<Self, ParseNonFungibleAddressError> {
        let v = s
            .splitn(2, ':')
            .filter(|&s| !s.is_empty())
            .collect::<Vec<&str>>();
        if v.len() != 2 {
            return Err(ParseNonFungibleAddressError::RequiresTwoParts);
        }
        let resource_address = bech32_decoder.validate_and_decode_resource_address(v[0])?;
        let non_fungible_id = NonFungibleId::try_from_combined_simple_string(v[1])?;
        Ok(NonFungibleAddress::new(resource_address, non_fungible_id))
    }
}

//======
// text
//======

impl fmt::Debug for NonFungibleAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

pub trait FromPublicKey: Sized {
    fn from_public_key<P: Into<PublicKey> + Clone>(public_key: &P) -> Self;
}

impl FromPublicKey for NonFungibleAddress {
    fn from_public_key<P: Into<PublicKey> + Clone>(public_key: &P) -> Self {
        let public_key: PublicKey = public_key.clone().into();
        match public_key {
            PublicKey::EcdsaSecp256k1(public_key) => NonFungibleAddress::new(
                ECDSA_SECP256K1_TOKEN,
                NonFungibleId::Bytes(hash(public_key.to_vec()).lower_26_bytes().into()),
            ),
            PublicKey::EddsaEd25519(public_key) => NonFungibleAddress::new(
                EDDSA_ED25519_TOKEN,
                NonFungibleId::Bytes(hash(public_key.to_vec()).lower_26_bytes().into()),
            ),
        }
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for NonFungibleAddress {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        write!(
            f,
            "{}:{}",
            self.resource_address.display(*context),
            self.non_fungible_id()
        )
    }
}

//======
// test
//======

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::Bech32Decoder;

    #[test]
    fn non_fungible_address_canonical_conversion() {
        let dec = Bech32Decoder::for_simulator();
        let enc = Bech32Encoder::for_simulator();

        assert_eq!(
            NonFungibleAddress::try_from_canonical_string(
                &dec,
                NonFungibleIdTypeId::Number,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:1",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:1"
        );

        assert_eq!(
            NonFungibleAddress::try_from_canonical_string(
                &dec,
                NonFungibleIdTypeId::Number,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:10",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:10"
        );

        assert_eq!(
            NonFungibleAddress::try_from_canonical_string(
                &dec,
                NonFungibleIdTypeId::UUID,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:1234567890",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:1234567890"
        );

        assert_eq!(
            NonFungibleAddress::try_from_canonical_string(
                &dec,
                NonFungibleIdTypeId::String,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:test",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:test"
        );

        assert_eq!(
            NonFungibleAddress::try_from_canonical_string(
                &dec,
                NonFungibleIdTypeId::Bytes,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:010a",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:010a"
        );
    }

    #[test]
    fn non_fungible_address_canonical_conversion_error() {
        let bech32_decoder = Bech32Decoder::for_simulator();
        assert_eq!(
            NonFungibleAddress::try_from_canonical_string(
                &bech32_decoder,
                NonFungibleIdTypeId::Number,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p",
            ),
            Err(ParseNonFungibleAddressError::RequiresTwoParts)
        );

        assert_eq!(
            NonFungibleAddress::try_from_canonical_string(
                &bech32_decoder,
                NonFungibleIdTypeId::String,
                // : is not currently allowed in non-fungible ids
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:1:2",
            ),
            Err(ParseNonFungibleAddressError::InvalidNonFungibleId(
                ParseNonFungibleIdError::InvalidCharacter(':')
            ))
        );

        assert_eq!(
            NonFungibleAddress::try_from_canonical_string(
                &bech32_decoder,
                NonFungibleIdTypeId::Number,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:",
            ),
            Err(ParseNonFungibleAddressError::RequiresTwoParts)
        );

        assert_eq!(
            NonFungibleAddress::try_from_canonical_string(
                &bech32_decoder,
                NonFungibleIdTypeId::Number,
                ":",
            ),
            Err(ParseNonFungibleAddressError::RequiresTwoParts)
        );

        assert!(matches!(
            NonFungibleAddress::try_from_canonical_string(
                &bech32_decoder,
                NonFungibleIdTypeId::Number,
                "3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:1",
            ),
            Err(ParseNonFungibleAddressError::InvalidResourceAddress(_))
        ));

        assert!(matches!(
            NonFungibleAddress::try_from_canonical_string(
                &bech32_decoder,
                NonFungibleIdTypeId::Number,
                "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:notnumber",
            ),
            Err(ParseNonFungibleAddressError::InvalidNonFungibleId(
                ParseNonFungibleIdError::InvalidInt(_)
            ))
        ));
    }
}
