use radix_engine_tests::common::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::{metadata, metadata_init};
use scrypto::NonFungibleData;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn package_burn_is_only_callable_within_resource_package() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                18,
                FungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    // Act
    let manifest = ManifestBuilder::new()
        .mint_fungible(resource_address, 10)
        .take_all_from_worktop(resource_address, "bucket")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                resource_address,
                RESOURCE_MANAGER_PACKAGE_BURN_IDENT,
                manifest_args!(lookup.bucket("bucket")),
            )
        })
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error);
}

#[test]
fn can_burn_by_amount_from_fungible_vault() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("vault"));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                18,
                FungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_fungible(resource_address, 100)
            .take_all_from_worktop(resource_address, "to_burn")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("to_burn")),
                )
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("50")))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_fungible_vault(vault_id).unwrap(),
        dec!("50")
    )
}

#[test]
fn can_burn_by_amount_from_non_fungible_vault() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("vault"));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "to_burn")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("to_burn")),
                )
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!(1)))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let (amount, _) = test_runner.inspect_non_fungible_vault(vault_id).unwrap();
    assert_eq!(amount, dec!(1))
}

#[test]
fn can_burn_by_ids_from_non_fungible_vault() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("vault"));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            component_address,
            "burn_ids",
            manifest_args!(btreeset![NonFungibleLocalId::integer(1)]),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!(1)
    );
}

#[test]
fn can_burn_by_amount_from_fungible_vault_with_an_access_rule() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("vault"));
    let (public_key, _, _) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                18,
                FungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_fungible(resource_address, 100)
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("50")))
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_fungible_vault(vault_id).unwrap(),
        dec!("50")
    )
}

#[test]
fn can_burn_by_amount_from_non_fungible_vault_with_an_access_rule() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("vault"));
    let (public_key, _, _) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!(1)))
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge]);

    // Assert
    receipt.expect_commit_success();
    let (amount, _) = test_runner.inspect_non_fungible_vault(vault_id).unwrap();
    assert_eq!(amount, dec!(1))
}

#[test]
fn can_burn_by_ids_from_non_fungible_vault_with_an_access_rule() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("vault"));
    let (public_key, _, _) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            component_address,
            "burn_ids",
            manifest_args!(btreeset![NonFungibleLocalId::integer(1)]),
        )
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!(1)
    );
}

#[test]
fn cant_burn_by_amount_from_fungible_vault_with_an_access_rule_that_is_not_fulfilled() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("vault"));
    let (public_key, _, _) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                18,
                FungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_fungible(resource_address, 100)
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("50")))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error);
    assert_eq!(
        test_runner.inspect_fungible_vault(vault_id).unwrap(),
        dec!("100")
    )
}

#[test]
fn cant_burn_by_amount_from_non_fungible_vault_with_an_access_rule_that_is_not_fulfilled() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("vault"));
    let (public_key, _, _) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!(1)))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error);
    let (amount, _) = test_runner.inspect_non_fungible_vault(vault_id).unwrap();
    assert_eq!(amount, dec!("2"))
}

#[test]
fn cant_burn_by_ids_from_non_fungible_vault_with_an_access_rule_that_is_not_fulfilled() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("vault"));
    let (public_key, _, _) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            component_address,
            "burn_ids",
            manifest_args!(btreeset![NonFungibleLocalId::integer(1)]),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(is_auth_unauthorized_error);
    assert_eq!(
        test_runner.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!("2")
    );
}

#[test]
fn can_burn_by_amount_from_fungible_vault_of_a_locked_down_resource() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("vault"));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                18,
                FungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_fungible(resource_address, 100)
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!("50")))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_fungible_vault(vault_id).unwrap(),
        dec!("50")
    )
}

#[test]
fn can_burn_by_amount_from_non_fungible_vault_of_a_locked_down_resource() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("vault"));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "burn_amount", manifest_args!(dec!(1)))
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    let (amount, _) = test_runner.inspect_non_fungible_vault(vault_id).unwrap();
    assert_eq!(amount, dec!(1))
}

#[test]
fn can_burn_by_ids_from_non_fungible_vault_of_a_locked_down_resource() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("vault"));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                metadata!(),
                Option::<BTreeMap<NonFungibleLocalId, EmptyStruct>>::None,
            )
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    let component_address = {
        let manifest = ManifestBuilder::new()
            .mint_non_fungible(
                resource_address,
                btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                ),
            )
            .take_all_from_worktop(resource_address, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "VaultBurn",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
            .expect_commit_success()
            .new_component_addresses()[0]
    };
    let vault_id = get_vault_id(&mut test_runner, component_address);

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            component_address,
            "burn_ids",
            manifest_args!(btreeset![NonFungibleLocalId::integer(1)]),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.inspect_non_fungible_vault(vault_id).unwrap().0,
        dec!(1)
    );
}

#[test]
fn can_burn_by_amount_from_fungible_account_vault() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                18,
                FungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Some(100.into()),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            account,
            "burn",
            manifest_args!(resource_address, dec!("50")),
        )
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.get_component_balance(account, resource_address),
        dec!("50")
    )
}

#[test]
fn can_burn_by_amount_from_non_fungible_account_vault() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Some(btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                )),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(account, "burn", manifest_args!(resource_address, dec!(1)))
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.get_component_balance(account, resource_address),
        dec!(1)
    )
}

#[test]
fn can_burn_by_ids_from_non_fungible_account_vault() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_account(false);
    let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
    let virtual_signature_rule = rule!(require(virtual_signature_badge.clone()));
    let resource_address = {
        let manifest = ManifestBuilder::new()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                NonFungibleResourceRoles::single_locked_rule(virtual_signature_rule),
                metadata!(),
                Some(btreemap!(
                    NonFungibleLocalId::integer(1) => EmptyStruct {},
                    NonFungibleLocalId::integer(2) => EmptyStruct {},
                )),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();
        test_runner
            .execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge.clone()])
            .expect_commit_success()
            .new_resource_addresses()[0]
    };

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            account,
            "burn_non_fungibles",
            manifest_args!(resource_address, indexset!(NonFungibleLocalId::integer(1))),
        )
        .build();
    let receipt =
        test_runner.execute_manifest_ignoring_fee(manifest, vec![virtual_signature_badge]);

    // Assert
    receipt.expect_commit_success();
    assert_eq!(
        test_runner.get_component_balance(account, resource_address),
        dec!(1)
    )
}

fn get_vault_id(
    test_runner: &mut DefaultTestRunner,
    component_address: ComponentAddress,
) -> NodeId {
    let manifest = ManifestBuilder::new()
        .call_method(component_address, "vault_id", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);
    receipt.expect_commit_success().output(1)
}

#[derive(NonFungibleData, ScryptoSbor, ManifestSbor)]
struct EmptyStruct {}

fn is_auth_unauthorized_error(error: &RuntimeError) -> bool {
    matches!(
        error,
        RuntimeError::SystemModuleError(SystemModuleError::AuthError(
            AuthError::Unauthorized { .. }
        ))
    )
}
