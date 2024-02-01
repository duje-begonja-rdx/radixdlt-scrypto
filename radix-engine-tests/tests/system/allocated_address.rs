use radix_engine_tests::common::*;
use radix_engine::{
    errors::{CannotGlobalizeError, KernelError, RuntimeError, SystemError},
    types::*,
};
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_create_and_return() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package = test_runner.publish_package_simple(PackageLoader::get("allocated_address"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "AllocatedAddressTest",
            "create_and_return",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::KernelError(KernelError::OrphanedNodes(_)))
    });
}

#[test]
fn test_create_and_pass_address() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package = test_runner.publish_package_simple(PackageLoader::get("allocated_address"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "AllocatedAddressTest",
            "create_and_pass_address",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_create_and_call() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package = test_runner.publish_package_simple(PackageLoader::get("allocated_address"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "AllocatedAddressTest",
            "create_and_call",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::SystemError(SystemError::NotAnObject))
    });
}

#[test]
fn test_create_and_consume_within_frame() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package = test_runner.publish_package_simple(PackageLoader::get("allocated_address"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "AllocatedAddressTest",
            "create_and_consume_within_frame",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_create_and_consume_with_mismatching_blueprint() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package = test_runner.publish_package_simple(PackageLoader::get("allocated_address"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "AllocatedAddressTest",
            "create_and_consume_with_mismatching_blueprint",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::CannotGlobalize(
                CannotGlobalizeError::InvalidBlueprintId
            ))
        )
    });
}

#[test]
fn test_create_and_consume_in_another_frame() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package = test_runner.publish_package_simple(PackageLoader::get("allocated_address"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "AllocatedAddressTest",
            "create_and_consume_in_another_frame",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_create_and_store_in_key_value_store() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package = test_runner.publish_package_simple(PackageLoader::get("allocated_address"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "AllocatedAddressTest",
            "create_and_store_in_key_value_store",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_create_and_store_in_metadata() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package = test_runner.publish_package_simple(PackageLoader::get("allocated_address"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "AllocatedAddressTest",
            "create_and_store_in_metadata",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
