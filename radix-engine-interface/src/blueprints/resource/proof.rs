use crate::blueprints::resource::*;
use crate::data::scrypto::model::Own;
use crate::data::scrypto::model::*;
use crate::data::scrypto::ScryptoCustomTypeKind;
use crate::data::scrypto::ScryptoCustomValueKind;
use crate::*;
use radix_engine_common::types::*;
use sbor::rust::collections::BTreeSet;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use sbor::*;

pub const PROOF_DROP_IDENT: &str = "Proof_drop";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ProofDropInput {
    pub proof: Proof,
}

pub type ProofDropOutput = ();

pub const PROOF_GET_AMOUNT_IDENT: &str = "Proof_get_amount";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ProofGetAmountInput {}

pub type ProofGetAmountOutput = Decimal;

pub const PROOF_GET_RESOURCE_ADDRESS_IDENT: &str = "Proof_get_resource_address";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ProofGetResourceAddressInput {}

pub type ProofGetResourceAddressOutput = ResourceAddress;

pub const PROOF_CLONE_IDENT: &str = "clone";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ProofCloneInput {}

pub type ProofCloneOutput = Proof;

/// The validation to be evaluated against a `Proof`.
///
/// TODO: Evaluate if we should have a ProofValidationBuilder to construct more complex validation modes.
pub enum ProofValidation {
    /// Specifies that the `Proof` should be validated against a single `ResourceAddress`.
    Contains(ResourceAddress),

    /// Specifies that the `Proof` should be validating for containing a specific `NonFungibleGlobalId`.
    ContainsNonFungible(NonFungibleGlobalId),

    /// Specifies that the `Proof` should be validated against a single resource address and a set of `NonFungibleLocalId`s
    /// to ensure that the `Proof` contains all of the NonFungibles in the set.
    ContainsNonFungibles(ResourceAddress, BTreeSet<NonFungibleLocalId>),

    /// Specifies that the `Proof` should be validated for the amount of resources that it contains.
    ContainsAmount(ResourceAddress, Decimal),

    /// Specifies that the `Proof` should have its resource address validated against a set of `ResourceAddress`es. If
    /// the `Proof`'s resource address belongs to the set, then its valid.
    ContainsAnyOf(BTreeSet<ResourceAddress>),
}

impl From<ResourceAddress> for ProofValidation {
    fn from(resource_address: ResourceAddress) -> Self {
        Self::Contains(resource_address)
    }
}

impl From<NonFungibleGlobalId> for ProofValidation {
    fn from(non_fungible_global_id: NonFungibleGlobalId) -> Self {
        Self::ContainsNonFungible(non_fungible_global_id)
    }
}

/// Represents an error when validating proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofValidationError {
    InvalidResourceAddress(ResourceAddress),
    ResourceAddressDoesNotBelongToList,
    DoesNotContainOneNonFungible,
    NonFungibleLocalIdNotFound,
    InvalidAmount(Decimal),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ProofValidationError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ProofValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// Stub
//========

/// Represents a proof
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Proof(pub Own);

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Proof {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        Own::value_kind()
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Proof {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.0.encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Proof {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        Own::decode_body_with_value_kind(decoder, value_kind).map(|o| Self(o))
    }
}

impl Describe<ScryptoCustomTypeKind> for Proof {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::well_known(
        crate::data::scrypto::well_known_scrypto_custom_types::OWN_PROOF_ID,
    );
}
