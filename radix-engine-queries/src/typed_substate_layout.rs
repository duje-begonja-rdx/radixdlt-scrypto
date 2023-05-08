use radix_engine::types::*;
use sbor::rust::prelude::*;

// Import and re-export these types so they are available easily with a single import
pub use radix_engine::blueprints::access_controller::*;
pub use radix_engine::blueprints::account::*;
pub use radix_engine::blueprints::clock::*;
pub use radix_engine::blueprints::epoch_manager::*;
pub use radix_engine::blueprints::package::*;
pub use radix_engine::blueprints::resource::*;
pub use radix_engine::system::node_modules::access_rules::*;
pub use radix_engine::system::node_modules::metadata::*;
pub use radix_engine::system::node_modules::royalty::*;
pub use radix_engine::system::node_modules::type_info::*;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
//
// The below defines well-known substate types which are used in the
// Core API of the node.
//
// Specifically:
// * Every (EntityType, SysModuleId, SubstateKey) should be mappable into a `TypedSubstateKey`
// * Every (&TypedSubstateKey, Data) should be mappable into a `WellKnownSubstateData`
//
// Please keep them these in-line with the well-known objects, and please don't
// remove these without talking to the Network team.
//=========================================================================

//=========================================================================
// TODO - move this to a relevant REP when it's been created
//
// BACKGROUND:
// A generic Object SysModule consists of 0 or more DbModules, each with a u8 "ModuleId".
//
// These modules are one of four types:
// - Tuple
//   => Has key: TupleKey(u8) also known as an offset
//   => No iteration exposed to engine
//   => Is versioned / locked substate-by-substate
// - ConcurrentMap
//   => Has key: MapKey(Vec<u8>)
//   => No iteration exposed to engine
//   => Is versioned / locked substate-by-substate
// - Index
//   => Has key: MapKey(Vec<u8>)
//   => Iteration exposed to engine via the MapKey's database key (ie hash of the key)
//   => Is versioned / locked in its entirety
// - SortedU16Index(SortedU16Key(U16, Vec<u8>))
//   => Has key: MapKey(Vec<u8>)
//   => Iteration exposed to engine via the user-controlled U16 prefix and then the MapKey's database key (ie hash of the key)
//   => Is versioned / locked in its entirety
//
// But in this file, we just handle explicitly supported/possible combinations of things.
//
// An entirely generic capturing of a substate type would look something like this:
//
// pub enum GenericObjectModuleSubstateType {
//    Tuple(ModuleId, TupleKey),
//    ConcurrentMap(ModuleId, MapKey),
//    Index(ModuleId, MapKey),
//    SortedU16Index(ModuleId, SortedU16Key),
// }
//=========================================================================

/// By node module (roughly SysModule)
#[derive(Debug, Clone)]
pub enum TypedSubstateKey {
    TypeInfoModule(TypeInfoField),
    AccessRulesModule(AccessRulesField),
    RoyaltyModule(RoyaltyField),
    MetadataModule(String),
    ObjectModule(TypedObjectModuleSubstateKey),
}

impl TypedSubstateKey {
    /// This method should be used to filter out substates which we don't want to map in the Core API.
    /// (See `radix-engine-tests/tests/bootstrap.rs` for an example of how it should be used)
    /// Just a work around for now to filter out "transient" substates we shouldn't really be storing.
    pub fn value_is_mappable(&self) -> bool {
        match self {
            TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::NonFungibleVault(
                NonFungibleVaultField::LockedNonFungible,
            )) => false,
            TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::FungibleVault(
                FungibleVaultField::LockedFungible,
            )) => false,
            _ => true,
        }
    }
}

/// Doesn't include non-object modules, nor transient nodes.
#[derive(Debug, Clone)]
pub enum TypedObjectModuleSubstateKey {
    // Objects
    Package(PackageField),
    FungibleResource(FungibleResourceManagerField),
    NonFungibleResourceField(NonFungibleResourceManagerField),
    NonFungibleResourceData(MapKey),
    FungibleVault(FungibleVaultField),
    NonFungibleVault(NonFungibleVaultField),
    NonFungibleVaultIndex(MapKey),
    EpochManagerField(EpochManagerField),
    EpochManagerSortedIndex(SortedU16Key),
    Clock(ClockField),
    Validator(ValidatorField),
    Account(MapKey),
    AccessController(AccessControllerField),
    // Generic Scrypto Components
    GenericScryptoComponent(ComponentField),
    // Substates for Generic KV Stores
    GenericKeyValueStore(MapKey), // Is an entity type with a single ConcurrentMap
}

fn error(descriptor: &'static str) -> String {
    format!("Could not convert {} to TypedSubstateKey", descriptor)
}

pub fn to_typed_substate_key(
    entity_type: EntityType,
    partition_num: PartitionNumber,
    substate_key: &SubstateKey,
) -> Result<TypedSubstateKey, String> {
    let substate_type = match partition_num {
        TYPE_INFO_FIELD_PARTITION => TypedSubstateKey::TypeInfoModule(
            TypeInfoField::try_from(substate_key).map_err(|_| error("TypeInfoOffset"))?,
        ),
        METADATA_KV_STORE_PARTITION => TypedSubstateKey::MetadataModule(
            scrypto_decode(
                substate_key
                    .for_map()
                    .ok_or_else(|| error("Metadata key"))?,
            )
            .map_err(|_| error("string Metadata key"))?,
        ),
        ROYALTY_FIELD_PARTITION => TypedSubstateKey::RoyaltyModule(
            RoyaltyField::try_from(substate_key).map_err(|_| error("RoyaltyOffset"))?,
        ),
        ACCESS_RULES_FIELD_PARTITION => TypedSubstateKey::AccessRulesModule(
            AccessRulesField::try_from(substate_key).map_err(|_| error("AccessRulesOffset"))?,
        ),
        partition_num @ _ if partition_num >= OBJECT_BASE_PARTITION => {
            TypedSubstateKey::ObjectModule(to_typed_object_module_substate_key(
                entity_type,
                partition_num.0 - OBJECT_BASE_PARTITION.0,
                substate_key,
            )?)
        }
        _ => return Err(format!("Unknown partition {:?}", partition_num)),
    };
    Ok(substate_type)
}

pub fn to_typed_object_module_substate_key(
    entity_type: EntityType,
    partition_offset: u8,
    substate_key: &SubstateKey,
) -> Result<TypedObjectModuleSubstateKey, String> {
    return to_typed_object_substate_key_internal(entity_type, partition_offset, substate_key)
        .map_err(|_| {
            format!(
                "Could not convert {:?} {:?} key to TypedObjectSubstateKey",
                entity_type, substate_key
            )
        });
}

fn to_typed_object_substate_key_internal(
    entity_type: EntityType,
    partition_offset: u8,
    substate_key: &SubstateKey,
) -> Result<TypedObjectModuleSubstateKey, ()> {
    let substate_type = match entity_type {
        EntityType::InternalGenericComponent | EntityType::GlobalGenericComponent => {
            TypedObjectModuleSubstateKey::GenericScryptoComponent(ComponentField::try_from(
                substate_key,
            )?)
        }
        EntityType::GlobalPackage => {
            TypedObjectModuleSubstateKey::Package(PackageField::try_from(substate_key)?)
        }
        EntityType::GlobalFungibleResource => TypedObjectModuleSubstateKey::FungibleResource(
            FungibleResourceManagerField::try_from(substate_key)?,
        ),
        EntityType::GlobalNonFungibleResource => {
            let partition_offset =
                NonFungibleResourceManagerPartitionOffset::try_from(partition_offset)?;
            match partition_offset {
                NonFungibleResourceManagerPartitionOffset::ResourceManager => {
                    TypedObjectModuleSubstateKey::NonFungibleResourceField(
                        NonFungibleResourceManagerField::try_from(substate_key)?,
                    )
                }
                NonFungibleResourceManagerPartitionOffset::NonFungibleData => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedObjectModuleSubstateKey::NonFungibleResourceData(key.clone())
                }
            }
        }
        EntityType::GlobalEpochManager => {
            let partition_offset = EpochManagerPartitionOffset::try_from(partition_offset)?;
            match partition_offset {
                EpochManagerPartitionOffset::EpochManager => {
                    TypedObjectModuleSubstateKey::EpochManagerField(EpochManagerField::try_from(
                        substate_key,
                    )?)
                }
                EpochManagerPartitionOffset::SecondaryIndex => {
                    let key = substate_key.for_sorted().ok_or(())?;
                    TypedObjectModuleSubstateKey::EpochManagerSortedIndex(key.clone())
                }
            }
        }
        EntityType::GlobalValidator => {
            TypedObjectModuleSubstateKey::Validator(ValidatorField::try_from(substate_key)?)
        }
        EntityType::GlobalClock => {
            TypedObjectModuleSubstateKey::Clock(ClockField::try_from(substate_key)?)
        }
        EntityType::GlobalAccessController => TypedObjectModuleSubstateKey::AccessController(
            AccessControllerField::try_from(substate_key)?,
        ),
        EntityType::GlobalVirtualSecp256k1Account
        | EntityType::GlobalVirtualEd25519Account
        | EntityType::InternalAccount
        | EntityType::GlobalAccount => {
            let key = substate_key.for_map().ok_or(())?;
            TypedObjectModuleSubstateKey::Account(key.clone())
        }
        EntityType::GlobalVirtualSecp256k1Identity
        | EntityType::GlobalVirtualEd25519Identity
        | EntityType::GlobalIdentity => Err(())?, // Identity doesn't have any substates
        EntityType::InternalFungibleVault => {
            TypedObjectModuleSubstateKey::FungibleVault(FungibleVaultField::try_from(substate_key)?)
        }
        EntityType::InternalNonFungibleVault => {
            let partition_offset = NonFungibleVaultPartitionOffset::try_from(partition_offset)?;

            match partition_offset {
                NonFungibleVaultPartitionOffset::Balance => {
                    TypedObjectModuleSubstateKey::NonFungibleVault(NonFungibleVaultField::try_from(
                        substate_key,
                    )?)
                }
                NonFungibleVaultPartitionOffset::NonFungibles => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedObjectModuleSubstateKey::NonFungibleVaultIndex(key.clone())
                }
            }
        }
        // These seem to be spread between Object and Virtualized SysModules
        EntityType::InternalKeyValueStore => {
            let key = substate_key.for_map().ok_or(())?;
            TypedObjectModuleSubstateKey::GenericKeyValueStore(key.clone())
        }
        EntityType::InternalIndex => {
            let key = substate_key.for_map().ok_or(())?;
            TypedObjectModuleSubstateKey::NonFungibleVaultIndex(key.clone())
        }
        EntityType::InternalSortedIndex => {
            let key = substate_key.for_sorted().ok_or(())?;
            TypedObjectModuleSubstateKey::EpochManagerSortedIndex(key.clone())
        }
    };
    Ok(substate_type)
}

// SysModuleId::Virtualized is currently a messy workaround / hodge-podge of different ideas and will be removed soon.
pub fn to_typed_virtualized_partition_substate_key(
    entity_type: EntityType,
    substate_key: &SubstateKey,
) -> Result<TypedSubstateKey, String> {
    let substate_type = match (entity_type, substate_key) {
        (EntityType::InternalKeyValueStore, SubstateKey::Map(key)) => {
            TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::GenericKeyValueStore(
                key.clone(),
            ))
        }
        (EntityType::InternalIndex, SubstateKey::Map(key)) => TypedSubstateKey::ObjectModule(
            TypedObjectModuleSubstateKey::NonFungibleVaultIndex(key.clone()),
        ),
        (EntityType::InternalSortedIndex, SubstateKey::Sorted(key)) => {
            TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::EpochManagerSortedIndex(
                key.clone(),
            ))
        }
        (_, SubstateKey::Map(key)) => {
            // For some reason, Metadata gets mapped under Virtualized SysModuleId on any entity type
            // But the good thing is that it's the only thing which is mapped under Virtualized SysModuleId for global components
            TypedSubstateKey::MetadataModule(
                scrypto_decode(key).map_err(|_| error("string Metadata key"))?,
            )
        }
        // Everything else is should be on the object substate
        _ => Err(format!(
            "Could not convert {:?} {:?} key to TypedObjectSubstateKey",
            entity_type, substate_key
        ))?,
    };
    Ok(substate_type)
}

#[derive(Debug, Clone)]
pub enum TypedSubstateValue {
    TypeInfoModule(TypedTypeInfoModuleSubstateValue),
    AccessRulesModule(TypedAccessRulesModuleSubstateValue),
    RoyaltyModule(TypedRoyaltyModuleSubstateValue),
    MetadataModule(TypedMetadataModuleSubstateValue),
    ObjectModule(TypedObjectModuleSubstateValue),
}

#[derive(Debug, Clone)]
pub enum TypedTypeInfoModuleSubstateValue {
    TypeInfo(TypeInfoSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedAccessRulesModuleSubstateValue {
    MethodAccessRules(MethodAccessRulesSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedRoyaltyModuleSubstateValue {
    ComponentRoyaltyConfig(ComponentRoyaltyConfigSubstate),
    ComponentRoyaltyAccumulator(ComponentRoyaltyAccumulatorSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedMetadataModuleSubstateValue {
    Metadata(MetadataValueSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedObjectModuleSubstateValue {
    // Objects
    Package(TypedPackageSubstateValue),
    FungibleResource(TypedFungibleResourceManagerSubstateValue),
    NonFungibleResource(TypedNonFungibleResourceManagerSubstateValue),
    NonFungibleResourceData(GenericScryptoSborPayload),
    FungibleVault(TypedFungibleVaultSubstateValue),
    NonFungibleVault(TypedNonFungibleVaultSubstateValue),
    EpochManager(TypedEpochManagerSubstateValue),
    Clock(TypedClockSubstateValue),
    Validator(TypedValidatorSubstateValue),
    Account(TypedAccountSubstateValue),
    AccessController(TypedAccessControllerSubstateValue),
    // Generic Scrypto Components
    GenericScryptoComponent(GenericScryptoComponentSubstateValue),
    // Substates for Generic KV Stores
    GenericKeyValueStore(GenericScryptoSborPayload),
    GenericIndex(GenericScryptoSborPayload),
    GenericSortedU16Index(GenericScryptoSborPayload),
}

#[derive(Debug, Clone)]
pub enum TypedPackageSubstateValue {
    Info(PackageInfoSubstate),
    CodeType(PackageCodeTypeSubstate),
    Code(PackageCodeSubstate),
    Royalty(PackageRoyaltySubstate),
    FunctionAccessRules(PackageFunctionAccessRulesSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedFungibleResourceManagerSubstateValue {
    Divisibility(FungibleResourceManagerDivisibilitySubstate),
    TotalSupply(FungibleResourceManagerTotalSupplySubstate),
}

#[derive(Debug, Clone)]
pub enum TypedNonFungibleResourceManagerSubstateValue {
    IdType(NonFungibleResourceManagerIdTypeSubstate),
    MutableFields(NonFungibleResourceManagerMutableFieldsSubstate),
    TotalSupply(NonFungibleResourceManagerTotalSupplySubstate),
}

#[derive(Debug, Clone)]
pub enum TypedFungibleVaultSubstateValue {
    Balance(FungibleVaultBalanceSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedNonFungibleVaultSubstateValue {
    Balance(NonFungibleVaultBalanceSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedEpochManagerSubstateValue {
    Config(EpochManagerConfigSubstate),
    EpochManager(EpochManagerSubstate),
    CurrentValidatorSet(CurrentValidatorSetSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedClockSubstateValue {
    CurrentTimeRoundedToMinutes(ClockSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedValidatorSubstateValue {
    Validator(ValidatorSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedAccountSubstateValue {
    Account(Option<Own>),
}

#[derive(Debug, Clone)]
pub enum TypedAccessControllerSubstateValue {
    AccessController(AccessControllerSubstate),
}

#[derive(Debug, Clone)]
pub enum GenericScryptoComponentSubstateValue {
    State(GenericScryptoSborPayload),
}

#[derive(Debug, Clone)]
pub struct GenericScryptoSborPayload {
    pub data: Vec<u8>,
}

pub fn to_typed_substate_value(
    substate_key: &TypedSubstateKey,
    data: &[u8],
) -> Result<TypedSubstateValue, String> {
    to_typed_substate_value_internal(substate_key, data).map_err(|err| {
        format!(
            "Error decoding substate data for key {:?} - {:?}",
            substate_key, err
        )
    })
}

fn to_typed_substate_value_internal(
    substate_key: &TypedSubstateKey,
    data: &[u8],
) -> Result<TypedSubstateValue, DecodeError> {
    let substate_value = match substate_key {
        TypedSubstateKey::TypeInfoModule(type_info_offset) => {
            TypedSubstateValue::TypeInfoModule(match type_info_offset {
                TypeInfoField::TypeInfo => {
                    TypedTypeInfoModuleSubstateValue::TypeInfo(scrypto_decode(data)?)
                }
            })
        }
        TypedSubstateKey::AccessRulesModule(access_rules_offset) => {
            TypedSubstateValue::AccessRulesModule(match access_rules_offset {
                AccessRulesField::AccessRules => {
                    TypedAccessRulesModuleSubstateValue::MethodAccessRules(scrypto_decode(data)?)
                }
            })
        }
        TypedSubstateKey::RoyaltyModule(royalty_offset) => {
            TypedSubstateValue::RoyaltyModule(match royalty_offset {
                RoyaltyField::RoyaltyConfig => {
                    TypedRoyaltyModuleSubstateValue::ComponentRoyaltyConfig(scrypto_decode(data)?)
                }
                RoyaltyField::RoyaltyAccumulator => {
                    TypedRoyaltyModuleSubstateValue::ComponentRoyaltyAccumulator(scrypto_decode(
                        data,
                    )?)
                }
            })
        }
        TypedSubstateKey::MetadataModule(_) => TypedSubstateValue::MetadataModule(
            TypedMetadataModuleSubstateValue::Metadata(scrypto_decode(data)?),
        ),
        TypedSubstateKey::ObjectModule(object_substate_key) => TypedSubstateValue::ObjectModule(
            to_typed_object_substate_value(object_substate_key, data)?,
        ),
    };
    Ok(substate_value)
}

fn to_typed_object_substate_value(
    substate_key: &TypedObjectModuleSubstateKey,
    data: &[u8],
) -> Result<TypedObjectModuleSubstateValue, DecodeError> {
    let substate_value = match substate_key {
        TypedObjectModuleSubstateKey::Package(offset) => {
            TypedObjectModuleSubstateValue::Package(match offset {
                PackageField::Info => TypedPackageSubstateValue::Info(scrypto_decode(data)?),
                PackageField::CodeType => {
                    TypedPackageSubstateValue::CodeType(scrypto_decode(data)?)
                }
                PackageField::Code => TypedPackageSubstateValue::Code(scrypto_decode(data)?),
                PackageField::Royalty => TypedPackageSubstateValue::Royalty(scrypto_decode(data)?),
                PackageField::FunctionAccessRules => {
                    TypedPackageSubstateValue::FunctionAccessRules(scrypto_decode(data)?)
                }
            })
        }
        TypedObjectModuleSubstateKey::FungibleResource(offset) => {
            TypedObjectModuleSubstateValue::FungibleResource(match offset {
                FungibleResourceManagerField::Divisibility => {
                    TypedFungibleResourceManagerSubstateValue::Divisibility(scrypto_decode(data)?)
                }
                FungibleResourceManagerField::TotalSupply => {
                    TypedFungibleResourceManagerSubstateValue::TotalSupply(scrypto_decode(data)?)
                }
            })
        }
        TypedObjectModuleSubstateKey::NonFungibleResourceField(offset) => {
            TypedObjectModuleSubstateValue::NonFungibleResource(match offset {
                NonFungibleResourceManagerField::IdType => {
                    TypedNonFungibleResourceManagerSubstateValue::IdType(scrypto_decode(data)?)
                }
                NonFungibleResourceManagerField::MutableFields => {
                    TypedNonFungibleResourceManagerSubstateValue::MutableFields(scrypto_decode(
                        data,
                    )?)
                }
                NonFungibleResourceManagerField::TotalSupply => {
                    TypedNonFungibleResourceManagerSubstateValue::TotalSupply(scrypto_decode(data)?)
                }
            })
        }
        TypedObjectModuleSubstateKey::NonFungibleResourceData(_) => {
            TypedObjectModuleSubstateValue::NonFungibleResourceData(GenericScryptoSborPayload {
                data: data.to_vec(),
            })
        }
        TypedObjectModuleSubstateKey::FungibleVault(offset) => {
            TypedObjectModuleSubstateValue::FungibleVault(match offset {
                FungibleVaultField::LiquidFungible => {
                    TypedFungibleVaultSubstateValue::Balance(scrypto_decode(data)?)
                }
                // This shouldn't be persistable - so use a bizarre (but temporary!) placeholder error code here!
                FungibleVaultField::LockedFungible => Err(DecodeError::InvalidCustomValue)?,
            })
        }
        TypedObjectModuleSubstateKey::NonFungibleVault(offset) => {
            TypedObjectModuleSubstateValue::NonFungibleVault(match offset {
                NonFungibleVaultField::LiquidNonFungible => {
                    TypedNonFungibleVaultSubstateValue::Balance(scrypto_decode(data)?)
                }
                // This shouldn't be persistable - so use a bizarre (but temporary!) placeholder error code here!
                NonFungibleVaultField::LockedNonFungible => Err(DecodeError::InvalidCustomValue)?,
            })
        }
        TypedObjectModuleSubstateKey::NonFungibleVaultIndex(_) => {
            TypedObjectModuleSubstateValue::GenericIndex(GenericScryptoSborPayload {
                data: data.to_vec(),
            })
        }
        TypedObjectModuleSubstateKey::EpochManagerField(offset) => {
            TypedObjectModuleSubstateValue::EpochManager(match offset {
                EpochManagerField::Config => {
                    TypedEpochManagerSubstateValue::Config(scrypto_decode(data)?)
                }
                EpochManagerField::EpochManager => {
                    TypedEpochManagerSubstateValue::EpochManager(scrypto_decode(data)?)
                }
                EpochManagerField::CurrentValidatorSet => {
                    TypedEpochManagerSubstateValue::CurrentValidatorSet(scrypto_decode(data)?)
                }
            })
        }
        TypedObjectModuleSubstateKey::Clock(offset) => {
            TypedObjectModuleSubstateValue::Clock(match offset {
                ClockField::CurrentTimeRoundedToMinutes => {
                    TypedClockSubstateValue::CurrentTimeRoundedToMinutes(scrypto_decode(data)?)
                }
            })
        }
        TypedObjectModuleSubstateKey::Validator(offset) => {
            TypedObjectModuleSubstateValue::Validator(match offset {
                ValidatorField::Validator => {
                    TypedValidatorSubstateValue::Validator(scrypto_decode(data)?)
                }
            })
        }
        TypedObjectModuleSubstateKey::Account(_) => TypedObjectModuleSubstateValue::Account(
            TypedAccountSubstateValue::Account(scrypto_decode(data)?),
        ),
        TypedObjectModuleSubstateKey::AccessController(offset) => {
            TypedObjectModuleSubstateValue::AccessController(match offset {
                AccessControllerField::AccessController => {
                    TypedAccessControllerSubstateValue::AccessController(scrypto_decode(data)?)
                }
            })
        }
        TypedObjectModuleSubstateKey::GenericScryptoComponent(offset) => {
            TypedObjectModuleSubstateValue::GenericScryptoComponent(match offset {
                ComponentField::State0 => {
                    GenericScryptoComponentSubstateValue::State(GenericScryptoSborPayload {
                        data: data.to_vec(),
                    })
                }
            })
        }
        TypedObjectModuleSubstateKey::GenericKeyValueStore(_) => {
            TypedObjectModuleSubstateValue::GenericKeyValueStore(GenericScryptoSborPayload {
                data: data.to_vec(),
            })
        }
        TypedObjectModuleSubstateKey::EpochManagerSortedIndex(_) => {
            TypedObjectModuleSubstateValue::GenericSortedU16Index(GenericScryptoSborPayload {
                data: data.to_vec(),
            })
        }
    };
    Ok(substate_value)
}