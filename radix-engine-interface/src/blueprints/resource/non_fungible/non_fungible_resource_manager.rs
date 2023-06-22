use crate::blueprints::resource::*;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::{Arbitrary, Result, Unstructured};
use radix_engine_common::prelude::*;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use radix_engine_interface::types::NonFungibleData;
use sbor::rust::collections::{BTreeMap, BTreeSet};
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::{generate_full_schema, LocalTypeIndex, TypeAggregator};

pub const NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT: &str = "NonFungibleResourceManager";

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT: &str = "create";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct NonFungibleResourceManagerCreateInput {
    pub id_type: NonFungibleIdType,
    pub track_total_supply: bool,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
}

pub type NonFungibleResourceManagerCreateOutput = ResourceAddress;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_with_initial_supply";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
    pub id_type: NonFungibleIdType,
    pub track_total_supply: bool,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
    pub entries: BTreeMap<NonFungibleLocalId, (ManifestValue,)>,
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerCreateWithInitialSupplyInput {
    pub id_type: NonFungibleIdType,
    pub track_total_supply: bool,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
    pub entries: BTreeMap<NonFungibleLocalId, (ScryptoValue,)>,
}

pub type NonFungibleResourceManagerCreateWithInitialSupplyOutput = (ResourceAddress, Bucket);

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT: &str =
    "create_non_fungible_with_address";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerCreateWithAddressInput {
    pub id_type: NonFungibleIdType,
    pub track_total_supply: bool,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
    pub resource_address: GlobalAddressReservation,
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerCreateWithAddressManifestInput {
    pub id_type: NonFungibleIdType,
    pub track_total_supply: bool,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
    pub resource_address: ManifestAddressReservation,
}

pub type NonFungibleResourceManagerCreateWithAddressOutput = ResourceAddress;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_ruid_non_fungible_with_initial_supply";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerCreateRuidWithInitialSupplyInput {
    pub track_total_supply: bool,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub metadata: BTreeMap<String, MetadataValue>,
    pub access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
    pub entries: Vec<(ScryptoValue,)>,
}

pub type NonFungibleResourceManagerCreateRuidWithInitialSupplyOutput = (ResourceAddress, Bucket);

pub const NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT: &str = "update_non_fungible_data";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerUpdateDataInput {
    pub id: NonFungibleLocalId,
    pub field_name: String,
    pub data: ScryptoValue,
}

pub type NonFungibleResourceManagerUpdateDataOutput = ();

pub const NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT: &str = "non_fungible_exists";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerExistsInput {
    pub id: NonFungibleLocalId,
}

pub type NonFungibleResourceManagerExistsOutput = bool;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT: &str = "get_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerGetNonFungibleInput {
    pub id: NonFungibleLocalId,
}

pub type NonFungibleResourceManagerGetNonFungibleOutput = ScryptoValue;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT: &str = "mint";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerMintManifestInput {
    pub entries: BTreeMap<NonFungibleLocalId, (ManifestValue,)>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMintInput {
    pub entries: BTreeMap<NonFungibleLocalId, (ScryptoValue,)>,
}

pub type NonFungibleResourceManagerMintOutput = Bucket;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT: &str = "mint_ruid";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerMintRuidManifestInput {
    pub entries: Vec<(ManifestValue,)>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMintRuidInput {
    pub entries: Vec<(ScryptoValue,)>,
}

pub type NonFungibleResourceManagerMintRuidOutput = Bucket;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_RUID_IDENT: &str = "mint_single_ruid";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMintSingleRuidInput {
    pub entry: ScryptoValue,
}
pub type NonFungibleResourceManagerMintSingleRuidOutput = (Bucket, NonFungibleLocalId);

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct NonFungibleDataSchema {
    pub schema: ScryptoSchema,
    pub non_fungible: LocalTypeIndex,
    pub mutable_fields: BTreeSet<String>,
}

impl NonFungibleData for () {
    const MUTABLE_FIELDS: &'static [&'static str] = &[];
}

impl NonFungibleDataSchema {
    pub fn new_schema<N: NonFungibleData>() -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let non_fungible_type = aggregator.add_child_type_and_descendents::<N>();
        let schema = generate_full_schema(aggregator);
        Self {
            schema,
            non_fungible: non_fungible_type,
            mutable_fields: N::MUTABLE_FIELDS.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn replace_self_package_address(&mut self, package_address: PackageAddress) {
        replace_self_package_address(&mut self.schema, package_address);
    }
}

#[cfg(feature = "radix_engine_fuzzing")]
impl<'a> Arbitrary<'a> for NonFungibleDataSchema {
    // At the moment I see no smart method to derive Arbitrary for type Schema, which is part of
    // ScryptoSchema, therefore implementing arbitrary by hand.
    // TODO: Introduce a method that genearates NonFungibleDataSchema in a truly random manner
    fn arbitrary(_u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(Self::new_schema::<()>())
    }
}
