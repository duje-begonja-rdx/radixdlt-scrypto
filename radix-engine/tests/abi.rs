#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use crate::ExpectedResult::{InvalidInput, InvalidOutput, Success};
use radix_engine::engine::RuntimeError;
use radix_engine::model::ComponentError;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn test_invalid_access_rule_methods() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("abi");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "AbiComponent",
            "create_invalid_abi_component",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let error = receipt.result.expect_err("Should be an error.");
    if !matches!(
        error,
        RuntimeError::ComponentError(ComponentError::BlueprintFunctionDoesNotExist(..))
    ) {
        panic!(
            "Should be an function does not exist but error was {}",
            error
        );
    }
}

enum ExpectedResult {
    Success,
    InvalidInput,
    InvalidOutput,
}

fn test_arg(method_name: &str, arg: Vec<u8>, expected_result: ExpectedResult) {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("abi");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "AbiComponent2", method_name, arg)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    match expected_result {
        ExpectedResult::Success => receipt.result.expect("Should be okay."),
        ExpectedResult::InvalidInput => {
            let error = receipt.result.expect_err("Should be an error.");
            if !matches!(error, RuntimeError::InvalidFnInput { .. }) {
                panic!("Error should be InvalidMethodArgument but was {:?}", error)
            }
        }
        ExpectedResult::InvalidOutput => {
            let error = receipt.result.expect_err("Should be an error.");
            if !matches!(error, RuntimeError::InvalidFnOutput { .. }) {
                panic!("Error should be InvalidMethodArgument but was {:?}", error)
            }
        }
    }
}

#[test]
fn test_invalid_output_fails() {
    test_arg("invalid_output", scrypto_encode(&()), InvalidOutput)
}

#[test]
fn test_input_arg_unit_succeeds() {
    test_arg("unit", scrypto_encode(&()), Success)
}

#[test]
fn test_invalid_input_arg_unit_fails() {
    test_arg("unit", scrypto_encode(&0u8), InvalidInput)
}

#[test]
fn test_input_arg_bool_succeeds() {
    test_arg("bool", scrypto_encode(&true), Success)
}

#[test]
fn test_invalid_input_arg_bool_fails() {
    test_arg("unit", scrypto_encode(&0u8), InvalidInput)
}

#[test]
fn test_input_arg_ivalue_succeeds() {
    test_arg("i8", scrypto_encode(&0i8), Success);
    test_arg("i16", scrypto_encode(&0i16), Success);
    test_arg("i32", scrypto_encode(&0i32), Success);
    test_arg("i64", scrypto_encode(&0i64), Success);
    test_arg("i128", scrypto_encode(&0i128), Success);
}

#[test]
fn test_input_arg_ivalue_fails() {
    test_arg("i8", scrypto_encode(&()), InvalidInput);
    test_arg("i16", scrypto_encode(&()), InvalidInput);
    test_arg("i32", scrypto_encode(&()), InvalidInput);
    test_arg("i64", scrypto_encode(&()), InvalidInput);
    test_arg("i128", scrypto_encode(&()), InvalidInput);
}

#[test]
fn test_input_arg_uvalue_succeeds() {
    test_arg("u8", scrypto_encode(&0u8), Success);
    test_arg("u16", scrypto_encode(&0u16), Success);
    test_arg("u32", scrypto_encode(&0u32), Success);
    test_arg("u64", scrypto_encode(&0u64), Success);
    test_arg("u128", scrypto_encode(&0u128), Success);
}

#[test]
fn test_input_arg_uvalue_fails() {
    test_arg("u8", scrypto_encode(&()), InvalidInput);
    test_arg("u16", scrypto_encode(&()), InvalidInput);
    test_arg("u32", scrypto_encode(&()), InvalidInput);
    test_arg("u64", scrypto_encode(&()), InvalidInput);
    test_arg("u128", scrypto_encode(&()), InvalidInput);
}

#[test]
fn test_input_arg_result_succeeds() {
    let okay: Result<(), ()> = Ok(());
    let error: Result<(), ()> = Err(());
    test_arg("result", scrypto_encode(&okay), Success);
    test_arg("result", scrypto_encode(&error), Success);
}

#[test]
fn test_invalid_input_arg_result_fails() {
    test_arg("result", scrypto_encode(&0u8), InvalidInput);
}

#[test]
fn test_input_arg_tree_map_succeeds() {
    let mut tree_map = BTreeMap::new();
    tree_map.insert((), ());
    test_arg("tree_map", scrypto_encode(&tree_map), Success);
}

#[test]
fn test_invalid_input_arg_tree_map_fails() {
    test_arg("tree_map", scrypto_encode(&0u8), InvalidInput);
}

#[test]
fn test_input_arg_hash_set_succeeds() {
    let mut hash_set = HashSet::new();
    hash_set.insert(());
    test_arg("hash_set", scrypto_encode(&hash_set), Success);
}

#[test]
fn test_invalid_input_arg_hash_set_fails() {
    test_arg("hash_set", scrypto_encode(&0u8), InvalidInput);
}