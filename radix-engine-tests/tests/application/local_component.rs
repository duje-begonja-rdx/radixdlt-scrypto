use radix_engine_tests::common::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::limits::TransactionLimitsError;
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn local_component_should_be_callable_read_only() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("local_component"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Secret",
            "read_local_component",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn local_component_should_be_callable_with_write() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("local_component"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Secret",
            "write_local_component",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn recursion_bomb() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("local_recursion"));

    // Act
    // Note: currently SEGFAULT occurs if bucket with too much in it is sent. My guess the issue is a native stack overflow.
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, XRD, Decimal::from(4u32))
        .take_all_from_worktop(XRD, "xrd")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "LocalRecursionBomb",
                "recursion_bomb",
                manifest_args!(lookup.bucket("xrd")),
            )
        })
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn recursion_bomb_to_failure() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("local_recursion"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, XRD, Decimal::from(100u32))
        .take_all_from_worktop(XRD, "bucket")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "LocalRecursionBomb",
                "recursion_bomb",
                manifest_args!(lookup.bucket("bucket")),
            )
        })
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
                TransactionLimitsError::MaxCallDepthLimitReached
            ))
        )
    });
}

#[test]
fn recursion_bomb_2() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("local_recursion"));

    // Act
    // Note: currently SEGFAULT occurs if bucket with too much in it is sent. My guess the issue is a native stack overflow.
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, XRD, Decimal::from(4u32))
        .take_all_from_worktop(XRD, "bucket")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "LocalRecursionBomb2",
                "recursion_bomb",
                manifest_args!(lookup.bucket("bucket")),
            )
        })
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn recursion_bomb_2_to_failure() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("local_recursion"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, XRD, Decimal::from(100u32))
        .take_all_from_worktop(XRD, "bucket")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "LocalRecursionBomb2",
                "recursion_bomb",
                manifest_args!(lookup.bucket("bucket")),
            )
        })
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::TransactionLimitsError(
                TransactionLimitsError::MaxCallDepthLimitReached
            ))
        )
    });
}
