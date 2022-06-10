#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::wasm::InvokeError;
use sbor::Type;
use scrypto::abi::BlueprintAbi;
use scrypto::prelude::{HashMap, Package, RADIX_TOKEN};
use scrypto::to_struct;
use test_runner::wat2wasm;
use transaction::builder::ManifestBuilder;

fn mocked_abi(blueprint_name: String) -> HashMap<String, BlueprintAbi> {
    let mut blueprint_abis = HashMap::new();
    blueprint_abis.insert(
        blueprint_name,
        BlueprintAbi {
            structure: Type::Unit,
            fns: Vec::new(),
        },
    );
    blueprint_abis
}

#[test]
fn test_loop() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000"));
    let package = Package {
        code,
        blueprints: mocked_abi("Test".to_string()),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_loop_out_of_cost_unit() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000000"));
    let package = Package {
        code,
        blueprints: mocked_abi("Test".to_string()),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    assert_invoke_error!(receipt.result, InvokeError::CostingError { .. })
}

#[test]
fn test_recursion() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    // In this test case, each call frame costs 4 stack units
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "128"));
    let package = Package {
        code,
        blueprints: mocked_abi("Test".to_string()),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_recursion_stack_overflow() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "129"));
    let package = Package {
        code,
        blueprints: mocked_abi("Test".to_string()),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    assert_invoke_error!(receipt.result, InvokeError::WasmError { .. })
}

#[test]
fn test_grow_memory() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100"));
    let package = Package {
        code,
        blueprints: mocked_abi("Test".to_string()),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_grow_memory_out_of_cost_unit() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100000"));
    let package = Package {
        code,
        blueprints: mocked_abi("Test".to_string()),
    };
    let package_address = test_runner.publish_package(package);
    let manifest = ManifestBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    assert_invoke_error!(receipt.result, InvokeError::CostingError { .. })
}

#[test]
fn test_total_cost_units_consumed() {
    // Arrange
    let mut test_runner = TestRunner::new(true);

    // Act
    let (pk1, _, account1) = test_runner.new_account();
    let (_, _, account2) = test_runner.new_account();
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(RADIX_TOKEN, account1)
        .call_method_with_all_resources(account2, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![pk1]);

    // Assert
    assert_eq!(237268, receipt.cost_units_consumed);
}
