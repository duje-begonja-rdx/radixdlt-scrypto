use radix_engine_tests::common::*;
use radix_engine::blueprints::resource::FungibleResourceManagerError;
use radix_engine::errors::{ApplicationError, KernelError, RuntimeError};
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn dangling_component_should_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("leaks"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Leaks",
            "dangling_component",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::KernelError(KernelError::OrphanedNodes(..)))
    });
}

#[test]
fn dangling_bucket_should_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("leaks"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Leaks",
            "dangling_bucket",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::KernelError(KernelError::OrphanedNodes(..)))
    });
}

#[test]
fn dangling_vault_should_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("leaks"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Leaks", "dangling_vault", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::KernelError(KernelError::OrphanedNodes(..)))
    });
}

#[test]
fn dangling_worktop_should_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("leaks"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Leaks", "get_bucket", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
                FungibleResourceManagerError::DropNonEmptyBucket
            ))
        )
    });
}

#[test]
fn dangling_kv_store_should_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("leaks"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Leaks",
            "dangling_kv_store",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::KernelError(KernelError::OrphanedNodes(..)))
    });
}

#[test]
fn dangling_bucket_with_proof_should_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("leaks"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Leaks",
            "dangling_bucket_with_proof",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::KernelError(KernelError::OrphanedNodes(..)))
    });
}
