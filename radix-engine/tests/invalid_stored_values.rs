#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn stored_bucket_in_committed_component_should_fail() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("stored_values");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "InvalidInitStoredBucket",
            "create",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::ValueNotAllowed));
}

#[test]
fn stored_bucket_in_owned_component_should_fail() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("stored_values");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .call_function(
            package_address,
            "InvalidStoredBucketInOwnedComponent",
            "create_bucket_in_owned_component",
            to_struct!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::ValueNotAllowed));
}
