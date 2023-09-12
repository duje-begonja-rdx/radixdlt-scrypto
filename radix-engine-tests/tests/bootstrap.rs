use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::bootstrap::*;
use radix_engine::system::checkers::SystemDatabaseChecker;
use radix_engine::system::checkers::{
    ResourceDatabaseChecker, ResourceEventChecker, ResourceReconciler, SystemEventChecker,
};
use radix_engine::system::system_db_reader::{ObjectCollectionKey, SystemDatabaseReader};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::transaction::{BalanceChange, CommitResult, SystemStructure};
use radix_engine::types::*;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::*;
use radix_engine_interface::api::node_modules::metadata::{MetadataValue, UncheckedUrl};
use radix_engine_queries::typed_substate_layout::*;
use radix_engine_store_interface::db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper};
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use scrypto_unit::{CustomGenesis, SubtreeVaults, TestRunnerBuilder};
use transaction::prelude::*;
use transaction::signing::secp256k1::Secp256k1PrivateKey;

#[test]
fn test_bootstrap_receipt_should_match_constants() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let vm = Vm::new(&scrypto_vm, native_vm);
    let mut substate_db = InMemorySubstateDatabase::standard();
    let validator_key = Secp256k1PublicKey([0; 33]);
    let staker_address = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    );
    let genesis_epoch = Epoch::of(1);
    let stake = GenesisStakeAllocation {
        account_index: 0,
        xrd_amount: Decimal::one(),
    };
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![validator_key.clone().into()]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_address],
            allocations: vec![(validator_key, vec![stake])],
        },
    ];

    let mut bootstrapper =
        Bootstrapper::new(NetworkDefinition::simulator(), &mut substate_db, vm, true);

    let GenesisReceipts {
        system_bootstrap_receipt,
        data_ingestion_receipts,
        wrap_up_receipt,
    } = bootstrapper
        .bootstrap_with_genesis_data(
            genesis_data_chunks,
            genesis_epoch,
            CustomGenesis::default_consensus_manager_config(),
            1,
            Some(0),
            Decimal::zero(),
        )
        .unwrap();

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_package_addresses()
        .contains(&PACKAGE_PACKAGE));

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_component_addresses()
        .contains(&GENESIS_HELPER));

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_package_addresses()
        .contains(&TRANSACTION_TRACKER_PACKAGE));

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_component_addresses()
        .contains(&TRANSACTION_TRACKER));

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_component_addresses()
        .contains(&FAUCET));

    let wrap_up_epoch_change = wrap_up_receipt
        .expect_commit_success()
        .next_epoch()
        .expect("There should be a new epoch.");

    assert_eq!(wrap_up_epoch_change.epoch, genesis_epoch.next().unwrap());

    let mut checker = SystemDatabaseChecker::<ResourceDatabaseChecker>::new();
    let db_results = checker
        .check_db(&substate_db)
        .expect("Database should be consistent");
    println!("{:#?}", db_results);

    let mut event_checker = SystemEventChecker::<ResourceEventChecker>::new();
    let mut events = Vec::new();
    events.push(
        system_bootstrap_receipt
            .expect_commit_success()
            .application_events
            .clone(),
    );
    events.extend(
        data_ingestion_receipts
            .into_iter()
            .map(|r| r.expect_commit_success().application_events.clone()),
    );
    events.push(
        wrap_up_receipt
            .expect_commit_success()
            .application_events
            .clone(),
    );
    let event_results = event_checker
        .check_all_events(&substate_db, &events)
        .expect("Events should be consistent");
    println!("{:#?}", event_results);

    ResourceReconciler::reconcile(&db_results.1, &event_results)
        .expect("Resource reconciliation failed.");
}

#[test]
fn test_bootstrap_receipts_should_have_complete_system_structure() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let vm = Vm::new(&scrypto_vm, native_vm);
    let mut substate_db = InMemorySubstateDatabase::standard();
    let validator_key = Secp256k1PublicKey([0; 33]);
    let staker_address = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    );
    let genesis_epoch = Epoch::of(1);
    let stake = GenesisStakeAllocation {
        account_index: 0,
        xrd_amount: Decimal::one(),
    };
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![validator_key.clone().into()]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_address],
            allocations: vec![(validator_key, vec![stake])],
        },
    ];

    let mut bootstrapper =
        Bootstrapper::new(NetworkDefinition::simulator(), &mut substate_db, vm, true);

    let GenesisReceipts {
        system_bootstrap_receipt,
        data_ingestion_receipts,
        wrap_up_receipt,
    } = bootstrapper
        .bootstrap_with_genesis_data(
            genesis_data_chunks,
            genesis_epoch,
            CustomGenesis::default_consensus_manager_config(),
            1,
            Some(0),
            Decimal::zero(),
        )
        .unwrap();

    assert_complete_system_structure(system_bootstrap_receipt.expect_commit_success());
    for data_ingestion_receipt in data_ingestion_receipts {
        assert_complete_system_structure(data_ingestion_receipt.expect_commit_success());
    }
    assert_complete_system_structure(wrap_up_receipt.expect_commit_success());
}

// TODO(after RCnet-V3): this assertion could be re-used for other tests of non-standard receipts.
fn assert_complete_system_structure(result: &CommitResult) {
    let SystemStructure {
        substate_system_structures,
        event_system_structures,
    } = &result.system_structure;

    let system_updates = result.state_updates.clone().into_legacy().system_updates;
    for ((node_id, partition_num), by_substate_key) in &system_updates {
        for substate_key in by_substate_key.keys() {
            let structure = substate_system_structures
                .get(node_id)
                .and_then(|partition_structures| partition_structures.get(partition_num))
                .and_then(|substate_structures| substate_structures.get(substate_key));
            assert!(
                structure.is_some(),
                "missing system structure for {:?}:{:?}:{:?}",
                node_id,
                partition_num,
                substate_key
            );
        }
    }

    for (event_type_id, _data) in &result.application_events {
        let structure = event_system_structures.get(event_type_id);
        assert!(
            structure.is_some(),
            "missing system structure for {:?}",
            event_type_id
        );
    }
}

fn test_genesis_resource_with_initial_allocation(owned_resource: bool) {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let vm = Vm::new(&scrypto_vm, native_vm);
    let mut substate_db = InMemorySubstateDatabase::standard();
    let token_holder = ComponentAddress::virtual_account_from_public_key(&PublicKey::Secp256k1(
        Secp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    ));
    let resource_address = ResourceAddress::new_or_panic(
        NodeId::new(
            EntityType::GlobalFungibleResourceManager as u8,
            &hash(vec![1, 2, 3]).lower_bytes(),
        )
        .0,
    );
    let resource_owner = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(2).unwrap().public_key(),
    );
    let allocation_amount = dec!("105");
    let genesis_resource = GenesisResource {
        reserved_resource_address: resource_address,
        metadata: vec![(
            "symbol".to_string(),
            MetadataValue::String("TST".to_string()),
        )],
        owner: if owned_resource {
            Some(resource_owner)
        } else {
            None
        },
    };
    let resource_allocation = GenesisResourceAllocation {
        account_index: 0,
        amount: allocation_amount,
    };
    let genesis_data_chunks = vec![
        GenesisDataChunk::Resources(vec![genesis_resource]),
        GenesisDataChunk::ResourceBalances {
            accounts: vec![token_holder.clone()],
            allocations: vec![(resource_address.clone(), vec![resource_allocation])],
        },
    ];

    let mut bootstrapper =
        Bootstrapper::new(NetworkDefinition::simulator(), &mut substate_db, vm, false);

    let GenesisReceipts {
        mut data_ingestion_receipts,
        ..
    } = bootstrapper
        .bootstrap_with_genesis_data(
            genesis_data_chunks,
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
            1,
            Some(0),
            Decimal::zero(),
        )
        .unwrap();

    let total_supply = substate_db
        .get_mapped::<SpreadPrefixKeyMapper, FungibleResourceManagerTotalSupplyFieldSubstate>(
            &resource_address.as_node_id(),
            MAIN_BASE_PARTITION,
            &FungibleResourceManagerField::TotalSupply.into(),
        )
        .unwrap()
        .into_payload()
        .into_latest();
    assert_eq!(total_supply, allocation_amount);

    let reader = SystemDatabaseReader::new(&substate_db);
    let entry = reader
        .read_object_collection_entry::<_, MetadataEntryEntryPayload>(
            resource_address.as_node_id(),
            ModuleId::Metadata,
            ObjectCollectionKey::KeyValue(
                MetadataCollection::EntryKeyValue.collection_index(),
                &"symbol".to_string(),
            ),
        )
        .unwrap()
        .map(|v| v.into_latest());

    if let Some(MetadataValue::String(symbol)) = entry {
        assert_eq!(symbol, "TST");
    } else {
        panic!("Resource symbol was not a string");
    }

    let allocation_receipt = data_ingestion_receipts.pop().unwrap();
    let resource_creation_receipt = data_ingestion_receipts.pop().unwrap();

    println!("{:?}", resource_creation_receipt);
    let resource_creation_commit = resource_creation_receipt.expect_commit_success();

    if owned_resource {
        let created_owner_badge = resource_creation_commit.new_resource_addresses()[1];
        let owner_badge_vault = resource_creation_commit.new_vault_addresses()[0];

        assert_eq!(
            resource_creation_commit
                .state_update_summary
                .vault_balance_changes
                .get(owner_badge_vault.as_node_id())
                .unwrap(),
            &(created_owner_badge, BalanceChange::Fungible(1.into()))
        );
    }

    let created_resource = resource_creation_commit.new_resource_addresses()[0]; // The resource address is preallocated, thus [0]
    let allocation_commit = allocation_receipt.expect_commit_success();
    let created_vault = allocation_commit.new_vault_addresses()[0];

    assert_eq!(
        allocation_commit
            .state_update_summary
            .vault_balance_changes
            .get(created_vault.as_node_id())
            .unwrap(),
        &(created_resource, BalanceChange::Fungible(allocation_amount))
    );
}

#[test]
fn test_genesis_resource_with_initial_owned_allocation() {
    test_genesis_resource_with_initial_allocation(true);
}

#[test]
fn test_genesis_resource_with_initial_unowned_allocation() {
    test_genesis_resource_with_initial_allocation(false);
}

#[test]
fn test_genesis_stake_allocation() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let vm = Vm::new(&scrypto_vm, native_vm);
    let mut substate_db = InMemorySubstateDatabase::standard();

    // There are two genesis validators
    // - one with two stakers (0 and 1)
    // - one with one staker (just 1)
    let validator_0_key = Secp256k1PrivateKey::from_u64(10).unwrap().public_key();
    let validator_1_key = Secp256k1PrivateKey::from_u64(11).unwrap().public_key();
    let staker_0 = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(4).unwrap().public_key(),
    );
    let staker_1 = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(5).unwrap().public_key(),
    );
    let validator_0_allocations = vec![
        GenesisStakeAllocation {
            account_index: 0,
            xrd_amount: dec!("10"),
        },
        GenesisStakeAllocation {
            account_index: 1,
            xrd_amount: dec!("50000"),
        },
    ];
    let validator_1_allocations = vec![GenesisStakeAllocation {
        account_index: 1,
        xrd_amount: dec!(1),
    }];
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![
            validator_0_key.clone().into(),
            validator_1_key.clone().into(),
        ]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_0, staker_1],
            allocations: vec![
                (validator_0_key, validator_0_allocations),
                (validator_1_key, validator_1_allocations),
            ],
        },
    ];

    let mut bootstrapper =
        Bootstrapper::new(NetworkDefinition::simulator(), &mut substate_db, vm, true);

    let GenesisReceipts {
        mut data_ingestion_receipts,
        ..
    } = bootstrapper
        .bootstrap_with_genesis_data(
            genesis_data_chunks,
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
            1,
            Some(0),
            Decimal::zero(),
        )
        .unwrap();

    let allocate_stakes_receipt = data_ingestion_receipts.pop().unwrap();

    let commit = allocate_stakes_receipt.expect_commit_success();
    let descendant_vaults = SubtreeVaults::new(&substate_db);

    // Staker 1 should have two liquidity balance entries
    {
        let address: GlobalAddress = staker_1.into();
        let balances = descendant_vaults
            .sum_balance_changes(address.as_node_id(), commit.vault_balance_changes());
        assert_eq!(balances.len(), 2);
        assert!(balances
            .values()
            .any(|bal| *bal == BalanceChange::Fungible(dec!(1))));
        assert!(balances
            .values()
            .any(|bal| *bal == BalanceChange::Fungible(dec!("50000"))));
    }

    let create_validators_receipt = data_ingestion_receipts.pop().unwrap();
    {
        let new_validators: Vec<ComponentAddress> = create_validators_receipt
            .expect_commit_success()
            .state_update_summary
            .new_components
            .iter()
            .filter(|c| c.as_node_id().entity_type() == Some(EntityType::GlobalValidator))
            .cloned()
            .collect();

        let reader = SystemDatabaseReader::new(&substate_db);

        for (index, validator_key) in vec![validator_0_key, validator_1_key]
            .into_iter()
            .enumerate()
        {
            let validator_url_entry = reader
                .read_object_collection_entry::<_, MetadataEntryEntryPayload>(
                    &new_validators[index].as_node_id(),
                    ModuleId::Metadata,
                    ObjectCollectionKey::KeyValue(
                        MetadataCollection::EntryKeyValue.collection_index(),
                        &"url".to_string(),
                    ),
                )
                .unwrap()
                .map(|v| v.into_latest());
            if let Some(MetadataValue::Url(url)) = validator_url_entry {
                assert_eq!(
                    url,
                    UncheckedUrl::of(format!("http://test.local?validator={:?}", validator_key))
                );
            } else {
                panic!("Validator url was not a Url");
            }
        }
    }
}

#[test]
fn test_genesis_time() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let vm = Vm::new(&scrypto_vm, native_vm);
    let mut substate_db = InMemorySubstateDatabase::standard();

    let mut bootstrapper =
        Bootstrapper::new(NetworkDefinition::simulator(), &mut substate_db, vm, false);

    let _ = bootstrapper
        .bootstrap_with_genesis_data(
            vec![],
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
            123 * 60 * 1000 + 22, // 123 full minutes + 22 ms (which should be rounded down)
            Some(0),
            Decimal::zero(),
        )
        .unwrap();

    let reader = SystemDatabaseReader::new(&mut substate_db);
    let timestamp = reader
        .read_typed_object_field::<ConsensusManagerProposerMinuteTimestampFieldPayload>(
            CONSENSUS_MANAGER.as_node_id(),
            ModuleId::Main,
            ConsensusManagerField::ProposerMinuteTimestamp.field_index(),
        )
        .unwrap()
        .into_latest();

    assert_eq!(timestamp.epoch_minute, 123);
}

#[test]
fn should_not_be_able_to_create_genesis_helper() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            GENESIS_HELPER_PACKAGE,
            GENESIS_HELPER_BLUEPRINT,
            "new",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn should_not_be_able_to_call_genesis_helper() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(GENESIS_HELPER, "wrap_up", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn mint_burn_events_should_match_resource_supply_post_genesis_and_notarized_tx() {
    // Arrange
    // Data migrated from Olympia
    let validator_0_key = Secp256k1PrivateKey::from_u64(10).unwrap().public_key();
    let validator_1_key = Secp256k1PrivateKey::from_u64(11).unwrap().public_key();
    let staker_0 = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(4).unwrap().public_key(),
    );
    let staker_1 = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(5).unwrap().public_key(),
    );
    let validator_0_allocations = vec![
        GenesisStakeAllocation {
            account_index: 0,
            xrd_amount: dec!("10"),
        },
        GenesisStakeAllocation {
            account_index: 1,
            xrd_amount: dec!("100"),
        },
    ];
    let validator_1_allocations = vec![GenesisStakeAllocation {
        account_index: 1,
        xrd_amount: dec!(2),
    }];
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![
            validator_0_key.clone().into(),
            validator_1_key.clone().into(),
        ]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_0, staker_1],
            allocations: vec![
                (validator_0_key, validator_0_allocations),
                (validator_1_key, validator_1_allocations),
            ],
        },
        GenesisDataChunk::XrdBalances(vec![(staker_0, dec!(200)), (staker_1, dec!(300))]),
    ];

    // Bootstrap
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_genesis(CustomGenesis {
            genesis_data_chunks: genesis_data_chunks,
            genesis_epoch: Epoch::of(1),
            initial_config: CustomGenesis::default_consensus_manager_config(),
            initial_time_ms: 0,
            initial_current_leader: Some(0),
            faucet_supply: *DEFAULT_TESTING_FAUCET_SUPPLY,
        })
        .build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .drop_auth_zone_proofs()
        .build();
    test_runner.execute_manifest(manifest, vec![]);

    // Assert
    println!("Staker 0: {:?}", staker_0);
    println!("Staker 1: {:?}", staker_1);
    let components = test_runner.find_all_components();
    let mut total_xrd_supply = Decimal::ZERO;
    for component in components {
        let xrd_balance = test_runner.get_component_balance(component, XRD);
        total_xrd_supply = total_xrd_supply.checked_add(xrd_balance).unwrap();
        println!("{:?}, {}", component, xrd_balance);
    }

    let mut total_mint_amount = Decimal::ZERO;
    let mut total_burn_amount = Decimal::ZERO;
    for tx_events in test_runner.collected_events() {
        for event in tx_events {
            match &event.0 .0 {
                Emitter::Method(x, _) if x.eq(XRD.as_node_id()) => {}
                _ => {
                    continue;
                }
            }
            let actual_type_name = test_runner.event_name(&event.0);
            match actual_type_name.as_str() {
                "MintFungibleResourceEvent" => {
                    total_mint_amount = total_mint_amount
                        .checked_add(
                            scrypto_decode::<MintFungibleResourceEvent>(&event.1)
                                .unwrap()
                                .amount,
                        )
                        .unwrap();
                }
                "BurnFungibleResourceEvent" => {
                    total_burn_amount = total_burn_amount
                        .checked_add(
                            scrypto_decode::<BurnFungibleResourceEvent>(&event.1)
                                .unwrap()
                                .amount,
                        )
                        .unwrap();
                }
                _ => {}
            }
        }
    }
    println!("Total XRD supply: {}", total_xrd_supply);
    println!("Total mint amount: {}", total_mint_amount);
    println!("Total burn amount: {}", total_burn_amount);
    assert_eq!(
        total_xrd_supply,
        total_mint_amount.checked_sub(total_burn_amount).unwrap()
    );
}
