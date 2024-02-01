use radix_engine_tests::common::*;
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn package_owner_can_claim_royalty() {
    // Arrange
    let (
        mut test_runner,
        account,
        public_key,
        package_address,
        _component_address,
        owner_badge_resource,
    ) = set_up_package_and_component();

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                [NonFungibleLocalId::integer(1)],
            )
            .claim_package_royalties(package_address)
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn non_package_owner_cannot_claim_royalty() {
    // Arrange
    let (
        mut test_runner,
        account,
        public_key,
        package_address,
        _component_address,
        _owner_badge_resource,
    ) = set_up_package_and_component();

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 5000)
            .claim_package_royalties(package_address)
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_failure();
}

#[test]
fn component_owner_can_set_royalty() {
    // Arrange
    let (
        mut test_runner,
        account,
        public_key,
        _package_address,
        component_address,
        owner_badge_resource,
    ) = set_up_package_and_component();

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                [NonFungibleLocalId::integer(1)],
            )
            .set_component_royalty(
                component_address,
                "paid_method".to_string(),
                RoyaltyAmount::Free,
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn non_component_owner_cannot_set_royalty() {
    // Arrange
    let (mut test_runner, account, public_key, _package_address, component_address, _) =
        set_up_package_and_component();

    // Negative case
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .set_component_royalty(
                component_address,
                "paid_method".to_string(),
                RoyaltyAmount::Free,
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_failure();
}

#[test]
fn component_owner_can_claim_royalty() {
    // Arrange
    let (
        mut test_runner,
        account,
        public_key,
        _package_address,
        component_address,
        owner_badge_resource,
    ) = set_up_package_and_component();

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                [NonFungibleLocalId::integer(1)],
            )
            .claim_component_royalties(component_address)
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn non_component_owner_cannot_claim_royalty() {
    // Arrange
    let (mut test_runner, account, public_key, _package_address, component_address, _) =
        set_up_package_and_component();

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 5000)
            .claim_component_royalties(component_address)
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_failure();
}

fn set_up_package_and_component() -> (
    DefaultTestRunner,
    ComponentAddress,
    Secp256k1PublicKey,
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
) {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let owner_badge_resource = test_runner.create_non_fungible_resource(account);
    let owner_badge_addr =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::integer(1));

    // Publish package
    let (code, definition) = PackageLoader::get("royalty-auth");
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .publish_package_with_owner(code, definition, owner_badge_addr.clone())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .create_proof_from_account_of_non_fungibles(
                account,
                owner_badge_resource,
                [NonFungibleLocalId::integer(1)],
            )
            .call_function(
                package_address,
                "RoyaltyTest",
                "create_component_with_royalty_enabled",
                manifest_args!(owner_badge_addr),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    (
        test_runner,
        account,
        public_key,
        package_address,
        component_address,
        owner_badge_resource,
    )
}
