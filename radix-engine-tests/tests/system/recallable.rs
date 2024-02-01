use radix_engine_tests::common::*;
use radix_engine::errors::{
    CallFrameError, KernelError, RejectionReason, RuntimeError, SystemModuleError,
};
use radix_engine::kernel::call_frame::{CreateFrameError, PassMessageError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::types::*;
use scrypto::prelude::FromPublicKey;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn non_existing_vault_should_cause_error() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();

    let non_existing_address = local_address(EntityType::InternalFungibleVault, 5);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .recall(non_existing_address, Decimal::one())
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_rejection(|e| {
        e.eq(&RejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(
            RuntimeError::KernelError(KernelError::InvalidReference(
                non_existing_address.as_node_id().clone(),
            )),
        ))
    });
}

#[test]
fn cannot_take_on_non_recallable_vault() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();

    let resource_address = test_runner.create_fungible_resource(10u32.into(), 0u8, account);
    let vaults = test_runner.get_component_vaults(account, resource_address);
    let vault_id = vaults[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .recall(
            InternalAddress::new_or_panic(vault_id.into()),
            Decimal::one(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                AuthError::Unauthorized { .. },
            ))
        )
    });
}

#[test]
fn can_take_on_recallable_vault() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();

    let recallable_token = test_runner.create_recallable_token(account);
    let vaults = test_runner.get_component_vaults(account, recallable_token);
    let vault_id = vaults[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .recall(
            InternalAddress::new_or_panic(vault_id.into()),
            Decimal::one(),
        )
        .try_deposit_entire_worktop_or_abort(other_account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();

    let original_account_amount = test_runner
        .get_component_resources(account)
        .get(&recallable_token)
        .cloned()
        .unwrap();
    let mut expected_amount: Decimal = 5u32.into();
    expected_amount = expected_amount.checked_sub(Decimal::one()).unwrap();
    assert_eq!(expected_amount, original_account_amount);

    let other_amount = test_runner
        .get_component_resources(other_account)
        .get(&recallable_token)
        .cloned()
        .unwrap();
    assert_eq!(other_amount, Decimal::one());
}

#[test]
fn test_recall_on_internal_vault() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.publish_package_simple(PackageLoader::get("recall"));

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_function(package_address, "RecallTest", "new", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let (component_address, _): (ComponentAddress, ResourceAddress) =
        receipt.expect_commit(true).output(1);

    // Recall
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32)
            .call_method(
                component_address,
                "recall_on_internal_vault",
                manifest_args!(),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::CreateFrameError(CreateFrameError::PassMessageError(
                    PassMessageError::DirectRefNotFound(..)
                ))
            ))
        )
    });
}

#[test]
fn test_recall_on_received_direct_access_reference() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let recallable_token_address = test_runner.create_recallable_token(account);
    let package_address = test_runner.publish_package_simple(PackageLoader::get("recall"));
    let vault_id = test_runner.get_component_vaults(account, recallable_token_address)[0];

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_function(
                package_address,
                "RecallTest",
                "recall_on_direct_access_ref",
                manifest_args!(InternalAddress::new_or_panic(vault_id.into())),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_recall_on_received_direct_access_reference_which_is_same_as_self() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    let package_address = test_runner.publish_package_simple(PackageLoader::get("recall"));
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
            .call_function(package_address, "RecallTest", "new", manifest_args!())
            .build(),
        vec![],
    );
    let (component_address, recallable): (ComponentAddress, ResourceAddress) =
        receipt.expect_commit(true).output(1);
    let vault_id = test_runner.get_component_vaults(component_address, recallable)[0];

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32)
            .call_method(
                component_address,
                "recall_on_direct_access_ref_method",
                manifest_args!(InternalAddress::new_or_panic(vault_id.into())),
            )
            .call_method(
                account,
                "try_deposit_batch_or_abort",
                manifest_args!(
                    ManifestExpression::EntireWorktop,
                    Option::<ResourceOrNonFungible>::None
                ),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}
