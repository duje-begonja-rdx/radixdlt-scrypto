use radix_engine_tests::common::*;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn stored_resource_is_invokeable() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package = test_runner.publish_package_simple(PackageLoader::get("stored_resource"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "StoredResource", "create", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let component = receipt.expect_commit(true).new_component_addresses()[0];

    // Act
    let manifest2 = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component, "total_supply", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest2, vec![]);

    // Assert
    receipt.expect_commit_success();
}
