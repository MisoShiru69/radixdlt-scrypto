#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn should_be_able_to_call_read_method_on_a_stored_component_in_owned_component() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("local_component");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "StoredSecret",
            "call_read_on_stored_component_in_owned_component",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn should_be_able_to_call_write_method_on_a_stored_component_in_owned_component() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("local_component");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "StoredSecret",
            "call_write_on_stored_component_in_owned_component",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn should_be_able_to_call_read_method_on_a_stored_component_in_global_component() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("local_component");
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "StoredSecret",
            "new_global",
            to_struct!(34567u32),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_success();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_method(component_address, "parent_get_secret", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let outputs = receipt.expect_success();
    let rtn: u32 = scrypto_decode(&outputs[0]).unwrap();
    assert_eq!(rtn, 34567u32);
}

#[test]
fn should_be_able_to_call_write_method_on_a_stored_component_in_global_component() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("local_component");
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "StoredSecret",
            "new_global",
            to_struct!(34567u32),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_success();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_method(component_address, "parent_set_secret", to_struct!(8888u32))
        .call_method(component_address, "parent_get_secret", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let outputs = receipt.expect_success();
    let rtn: u32 = scrypto_decode(&outputs[1]).unwrap();
    assert_eq!(rtn, 8888u32);
}

#[test]
fn should_be_able_to_call_read_method_on_a_kv_stored_component_in_owned_component() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("local_component");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "StoredKVLocal",
            "call_read_on_stored_component_in_owned_component",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn should_be_able_to_call_write_method_on_a_kv_stored_component_in_owned_component() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("local_component");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "StoredKVLocal",
            "call_write_on_stored_component_in_owned_component",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn should_be_able_to_call_read_method_on_a_kv_stored_component_in_global_component() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("local_component");
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "StoredKVLocal",
            "new_global",
            to_struct!(34567u32),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_success();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_method(component_address, "parent_get_secret", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let outputs = receipt.expect_success();
    let rtn: u32 = scrypto_decode(&outputs[0]).unwrap();
    assert_eq!(rtn, 34567u32);
}

#[test]
fn should_be_able_to_call_write_method_on_a_kv_stored_component_in_global_component() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("local_component");
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "StoredKVLocal",
            "new_global",
            to_struct!(34567u32),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_success();
    let component_address = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_method(component_address, "parent_set_secret", to_struct!(8888u32))
        .call_method(component_address, "parent_get_secret", to_struct!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let outputs = receipt.expect_success();
    let rtn: u32 = scrypto_decode(&outputs[1]).unwrap();
    assert_eq!(rtn, 8888u32);
}
