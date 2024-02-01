use radix_engine::{
    errors::{RuntimeError, SystemError},
    system::system_modules::costing::{
        NATIVE_FUNCTION_BASE_COSTS, NATIVE_FUNCTION_BASE_COSTS_SIZE_DEPENDENT,
    },
    types::*,
};
use radix_engine_interface::schema::TypeRef;
use radix_engine_queries::typed_substate_layout::{AccountNativePackage, BlueprintPayloadDef};
use radix_engine_tests::common::*;
use sbor::basic_well_known_types::*;
use scrypto_test::prelude::*;
use scrypto_unit::*;

#[test]
fn check_native_function_base_costs() {
    let test_runner = TestRunnerBuilder::new().build();
    let mut lookup: IndexMap<PackageAddress, IndexSet<String>> = index_map_new();
    let package_addresses = test_runner.find_all_packages();
    for package_address in package_addresses {
        let blueprint_definitions = test_runner.get_package_blueprint_definitions(&package_address);
        for (_, definition) in blueprint_definitions {
            let functions = definition.interface.functions;
            for (name, _) in functions {
                let export_name = definition
                    .function_exports
                    .get(&name)
                    .unwrap()
                    .export_name
                    .clone();
                lookup
                    .entry(package_address)
                    .or_default()
                    .insert(export_name);
            }
        }
    }

    for (package_address, m) in NATIVE_FUNCTION_BASE_COSTS.iter() {
        for (export_name, _) in m {
            if !matches!(
                lookup
                    .get(package_address)
                    .map(|x| x.contains(&export_name.to_string())),
                Some(true)
            ) {
                println!(
                    "Invalid definition: {}, {}",
                    package_address.to_hex(),
                    export_name
                );
            }
        }
    }

    println!();
    let mut missing_functions = false;

    for (package_address, m) in &lookup {
        for export_name in m {
            if !matches!(
                NATIVE_FUNCTION_BASE_COSTS
                    .get(package_address)
                    .map(|x| x.contains_key(export_name.as_str())),
                Some(true)
            ) && !matches!(
                NATIVE_FUNCTION_BASE_COSTS_SIZE_DEPENDENT
                    .get(package_address)
                    .map(|x| x.contains_key(export_name.as_str())),
                Some(true)
            ) && *package_address != FAUCET_PACKAGE
                && *package_address != GENESIS_HELPER_PACKAGE
            {
                println!(
                    "Missing definition: {}, {}",
                    package_address.to_hex(),
                    export_name
                );
                missing_functions = true;
            }
        }
    }

    println!();

    // In case of failing see: https://radixdlt.atlassian.net/wiki/spaces/S/pages/3042115875/Running+CPU+costing+evaluation
    assert!(!missing_functions);
}

#[test]
fn scan_native_blueprint_schemas_and_highlight_unsafe_types() {
    let test_runner = TestRunnerBuilder::new().build();
    let bech32 = AddressBech32Encoder::for_simulator();

    let package_addresses = test_runner.find_all_packages();
    for package_address in package_addresses {
        println!("\nChecking {}", package_address.to_string(&bech32));

        let schemas_by_hash = test_runner.get_package_scrypto_schemas(&package_address);
        println!("Found {} schemas", schemas_by_hash.len());

        let blueprint_definitions = test_runner.get_package_blueprint_definitions(&package_address);
        for (key, definition) in blueprint_definitions {
            println!("Checking blueprint {:?}", key.blueprint);
            if let Some(fields) = definition.interface.state.fields {
                for (i, f) in fields.1.iter().enumerate() {
                    let result = check_payload_def(&schemas_by_hash, &f.field);
                    if result.is_not_safe() {
                        println!("Field {:?} is {:?}", i, result);
                    }
                }
            }
            let collections = definition.interface.state.collections;
            for (partition, collection_schema) in collections {
                match collection_schema {
                    BlueprintCollectionSchema::KeyValueStore(kv) => {
                        let result = check_payload_defs(&schemas_by_hash, &[kv.key, kv.value]);
                        if result.is_not_safe() {
                            println!("Partition {:?} is {:?}", partition, result);
                        }
                    }
                    BlueprintCollectionSchema::Index(kv) => {
                        let result = check_payload_defs(&schemas_by_hash, &[kv.key, kv.value]);
                        if result.is_not_safe() {
                            println!("Partition {:?} is {:?}", partition, result);
                        }
                    }
                    BlueprintCollectionSchema::SortedIndex(kv) => {
                        let result = check_payload_defs(&schemas_by_hash, &[kv.key, kv.value]);
                        if result.is_not_safe() {
                            println!("Partition {:?} is {:?}", partition, result);
                        }
                    }
                }
            }
            let functions = definition.interface.functions;
            for (name, func) in functions {
                let result = check_payload_defs(&schemas_by_hash, &[func.input, func.output]);
                if result.is_not_safe() {
                    println!("Function {:?} is {:?}", name, result);
                }
            }
            let events = definition.interface.events;
            for (name, ty) in events {
                let result = check_payload_def(&schemas_by_hash, &ty);
                if result.is_not_safe() {
                    println!("Event {:?} is {:?}", name, result);
                }
            }
        }
    }
}

fn check_payload_defs(
    schemas_by_hash: &IndexMap<SchemaHash, VersionedScryptoSchema>,
    type_pointers: &[BlueprintPayloadDef],
) -> CheckResult {
    for ty in type_pointers {
        let result = check_payload_def(schemas_by_hash, ty);
        if result.is_not_safe() {
            return result;
        }
    }
    return CheckResult::Safe;
}

fn check_payload_def(
    schemas_by_hash: &IndexMap<SchemaHash, VersionedScryptoSchema>,
    type_pointer: &BlueprintPayloadDef,
) -> CheckResult {
    match type_pointer {
        BlueprintPayloadDef::Static(type_identifier) => check_type(
            schemas_by_hash.get(&type_identifier.0).unwrap(),
            type_identifier.1,
        ),
        BlueprintPayloadDef::Generic(_) => CheckResult::Safe,
    }
}

fn check_type(schema: &VersionedScryptoSchema, type_id: LocalTypeId) -> CheckResult {
    let mut visited_indices = index_set_new();
    check_type_internal(schema, type_id, &mut visited_indices)
}

fn check_types_internal(
    schema: &VersionedScryptoSchema,
    indices: &[LocalTypeId],
    visited_indices: &mut IndexSet<LocalTypeId>,
) -> CheckResult {
    for index in indices {
        let result = check_type_internal(schema, *index, visited_indices);
        if result.is_not_safe() {
            return result;
        }
    }
    CheckResult::Safe
}

fn check_type_internal(
    schema: &VersionedScryptoSchema,
    type_id: LocalTypeId,
    visited_indices: &mut IndexSet<LocalTypeId>,
) -> CheckResult {
    if visited_indices.contains(&type_id) {
        return CheckResult::Safe;
    }
    visited_indices.insert(type_id);
    match type_id {
        LocalTypeId::WellKnown(x) => return is_safe_well_known_type(schema, x),
        LocalTypeId::SchemaLocalIndex(i) => {
            let type_kind = &schema.v1().type_kinds[i];
            match type_kind {
                ScryptoTypeKind::Array { element_type } => {
                    return check_type_internal(schema, *element_type, visited_indices);
                }
                ScryptoTypeKind::Tuple { field_types } => {
                    return check_types_internal(schema, field_types, visited_indices);
                }
                ScryptoTypeKind::Enum { variants } => {
                    let mut indices = Vec::<LocalTypeId>::new();
                    for v in variants {
                        for ty in v.1 {
                            indices.push(*ty);
                        }
                    }
                    return check_types_internal(schema, &indices, visited_indices);
                }
                ScryptoTypeKind::Map {
                    key_type,
                    value_type,
                } => {
                    return check_types_internal(
                        schema,
                        &[*key_type, *value_type],
                        visited_indices,
                    );
                }
                ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Own) => {
                    match &schema.v1().type_validations[i] {
                        TypeValidation::Custom(ScryptoCustomTypeValidation::Own(x)) => match x {
                            OwnValidation::IsTypedObject(_, _) => {
                                return CheckResult::Safe;
                            }
                            OwnValidation::IsKeyValueStore => {
                                return CheckResult::PossiblyUnsafe {
                                    type_kind: type_kind.clone(),
                                    type_validation: schema.v1().type_validations[i].clone(),
                                };
                            }
                            OwnValidation::IsGlobalAddressReservation => {
                                return CheckResult::Safe;
                            }
                            _ => {
                                return CheckResult::PossiblyUnsafe {
                                    type_kind: type_kind.clone(),
                                    type_validation: schema.v1().type_validations[i].clone(),
                                };
                            }
                        },
                        _ => panic!("Wrong type validation attached to `Own` type kind"),
                    }
                }
                ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference) => {
                    match &schema.v1().type_validations[i] {
                        TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(x)) => {
                            match x {
                                ReferenceValidation::IsGlobalTyped(_, _)
                                | ReferenceValidation::IsInternalTyped(_, _)
                                | ReferenceValidation::IsGlobalPackage
                                | ReferenceValidation::IsGlobalResourceManager
                                | ReferenceValidation::IsGlobalComponent => {
                                    return CheckResult::Safe;
                                }
                                _ => {
                                    return CheckResult::PossiblyUnsafe {
                                        type_kind: type_kind.clone(),
                                        type_validation: schema.v1().type_validations[i].clone(),
                                    };
                                }
                            }
                        }
                        _ => panic!("Wrong type validation attached to `Reference` type kind"),
                    }
                }
                _ => {
                    return CheckResult::Safe;
                }
            }
        }
    };
}

fn is_safe_well_known_type(
    schema: &VersionedScryptoSchema,
    type_id: WellKnownTypeId,
) -> CheckResult {
    let is_safe = match type_id {
        // Basic SBOR
        BOOL_TYPE => true,
        I8_TYPE => true,
        I16_TYPE => true,
        I32_TYPE => true,
        I64_TYPE => true,
        I128_TYPE => true,
        U8_TYPE => true,
        U16_TYPE => true,
        U32_TYPE => true,
        U64_TYPE => true,
        U128_TYPE => true,
        STRING_TYPE => true,
        ANY_TYPE => false,
        BYTES_TYPE => true,
        UNIT_TYPE => true,

        // Scrypto SBOR
        REFERENCE_TYPE => false,
        GLOBAL_ADDRESS_TYPE => true, // TODO: maybe unsafe
        INTERNAL_ADDRESS_TYPE => false,
        PACKAGE_ADDRESS_TYPE => true,
        COMPONENT_ADDRESS_TYPE => true,
        RESOURCE_ADDRESS_TYPE => true,
        OWN_TYPE => false,
        OWN_BUCKET_TYPE => true, // TODO: maybe unsafe?
        OWN_FUNGIBLE_BUCKET_TYPE => true,
        OWN_NON_FUNGIBLE_BUCKET_TYPE => true,
        OWN_PROOF_TYPE => true, // TODO: maybe unsafe?
        OWN_FUNGIBLE_PROOF_TYPE => true,
        OWN_NON_FUNGIBLE_PROOF_TYPE => true,
        OWN_VAULT_TYPE => false,
        OWN_FUNGIBLE_VAULT_TYPE => true,
        OWN_NON_FUNGIBLE_VAULT_TYPE => true,
        OWN_KEY_VALUE_STORE_TYPE => true, // TODO: maybe unsafe?
        OWN_GLOBAL_ADDRESS_RESERVATION_TYPE => true,
        DECIMAL_TYPE => true,
        PRECISE_DECIMAL_TYPE => true,
        NON_FUNGIBLE_LOCAL_ID_TYPE => true,
        NON_FUNGIBLE_GLOBAL_ID_TYPE => true,
        INSTANT_TYPE => true,
        UTC_DATE_TIME_TYPE => true,
        URL_TYPE => true,
        ORIGIN_TYPE => true,
        PUBLIC_KEY_TYPE => true,
        SECP256K1_PUBLIC_KEY_TYPE => true,
        ED25519_PUBLIC_KEY_TYPE => true,
        PUBLIC_KEY_HASH_TYPE => true,
        SECP256K1_PUBLIC_KEY_HASH_TYPE => true,
        ED25519_PUBLIC_KEY_HASH_TYPE => true,
        ACCESS_RULE_TYPE => true,
        ACCESS_RULE_NODE_TYPE => true,
        ACCESS_RULE_NODE_LIST_TYPE => true,
        PROOF_RULE_TYPE => true,
        RESOURCE_OR_NON_FUNGIBLE_TYPE => true,
        RESOURCE_OR_NON_FUNGIBLE_LIST_TYPE => true,
        OWNER_ROLE_TYPE => true,
        ROLE_KEY_TYPE => true,
        MODULE_ID_TYPE => true,
        ATTACHED_MODULE_ID_TYPE => true,
        ROYALTY_AMOUNT_TYPE => true,
        t => panic!("Unexpected well-known type id: {:?}", t),
    };

    if is_safe {
        CheckResult::Safe
    } else {
        CheckResult::PossiblyUnsafe {
            type_kind: schema
                .v1()
                .resolve_type_kind(LocalTypeId::WellKnown(type_id))
                .unwrap()
                .clone(),
            type_validation: schema
                .v1()
                .resolve_type_validation(LocalTypeId::WellKnown(type_id))
                .unwrap()
                .clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CheckResult {
    Safe,
    PossiblyUnsafe {
        type_kind: ScryptoTypeKind<LocalTypeId>,
        type_validation: TypeValidation<ScryptoCustomTypeValidation>,
    },
}

impl CheckResult {
    fn is_safe(&self) -> bool {
        matches!(self, CheckResult::Safe)
    }
    fn is_not_safe(&self) -> bool {
        !self.is_safe()
    }
}

#[test]
pub fn test_fake_bucket() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let (code, mut definition) = PackageLoader::get("fake_bucket");
    definition
        .blueprints
        .get_mut("FakeBucket")
        .unwrap()
        .schema
        .state
        .fields[0]
        .field = TypeRef::Static(LocalTypeId::WellKnown(DECIMAL_TYPE));
    let package_address =
        test_runner.publish_package((code, definition), BTreeMap::new(), OwnerRole::None);

    // Test abusing vault put method
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .withdraw_from_account(account, XRD, 100)
            .take_from_worktop(XRD, 100, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "FakeBucket",
                    "free_1000_xrd",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemError(SystemError::TypeCheckError(e))
            if format!("{:?}", e).contains("Expected = Own<IsBucket>") =>
        {
            true
        }
        _ => false,
    });
}

#[test]
fn native_blueprints_with_typed_addresses_have_expected_schema() {
    let mut blueprint_definition = AccountNativePackage::definition()
        .blueprints
        .remove("Account")
        .unwrap();
    let TypeRef::Static(local_type_index) = blueprint_definition
        .schema
        .functions
        .functions
        .remove("create_advanced")
        .unwrap()
        .output
    else {
        panic!("Generic output!")
    };

    let schema = blueprint_definition.schema.schema.into_latest();
    let type_kind = schema.resolve_type_kind(local_type_index).unwrap();
    let type_validation = schema.resolve_type_validation(local_type_index).unwrap();

    assert!(matches!(
        type_kind,
        TypeKind::<ScryptoCustomTypeKind, LocalTypeId>::Custom(ScryptoCustomTypeKind::Reference)
    ));
    assert!(matches!(
        type_validation,
        TypeValidation::<ScryptoCustomTypeValidation>::Custom(
            ScryptoCustomTypeValidation::Reference(ReferenceValidation::IsGlobalTyped(
                Some(ACCOUNT_PACKAGE),
                bp_name
            ))
        ) if bp_name == "Account"
    ));
}
