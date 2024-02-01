use radix_engine_tests::common::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::limits::TransactionLimitsError;
use radix_engine::types::*;
use radix_engine_common::constants::MAX_CALL_DEPTH;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_max_call_depth_success() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("recursion"));

    // Act
    // ============================
    // Stack layout:
    // * 0: Executor
    // * 1: Transaction Executor
    // * 2-15: Caller::call x 14
    // ============================
    let num_calls = u32::try_from(MAX_CALL_DEPTH).unwrap() - 1u32;
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Caller",
            "recursive",
            manifest_args!(num_calls),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_max_call_depth_failure() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("recursion"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Caller",
            "recursive",
            manifest_args!(16u32),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

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
