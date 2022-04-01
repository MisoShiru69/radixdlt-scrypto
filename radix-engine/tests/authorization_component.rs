#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::errors::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;

#[test]
fn cannot_make_cross_component_call_without_authorization() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, _, account) = test_runner.new_account();
    let auth = test_runner.create_non_fungible_resource(account.clone());
    let auth_id = NonFungibleId::from(1);
    let auth_address = NonFungibleAddress::new(auth, auth_id);
    let method_authorization = component_authorization! {
        "get_component_state" => this!(auth_address.clone())
    };

    let package_address = test_runner.publish_package("component");
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_address,
            "CrossComponent",
            "create_component_with_auth",
            vec![scrypto_encode(&method_authorization)],
        )
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);
    receipt.result.expect("Should be okay");
    let secured_component = receipt.new_component_addresses[0];

    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_address,
            "CrossComponent",
            "create_component",
            vec![],
        )
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);
    assert!(receipt.result.is_ok());
    let my_component = receipt.new_component_addresses[0];

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(
            my_component,
            "cross_component_call",
            vec![scrypto_encode(&secured_component)],
        )
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be error");
    assert_eq!(runtime_error, RuntimeError::NotAuthorized);
}

#[test]
fn can_make_cross_component_call_with_authorization() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(&mut substate_store);
    let (_, _, account) = test_runner.new_account();
    let auth = test_runner.create_non_fungible_resource(account.clone());
    let auth_id = NonFungibleId::from(1);
    let auth_address = NonFungibleAddress::new(auth, auth_id.clone());
    let method_authorization = component_authorization! {
        "get_component_state" => this!(auth_address.clone())
    };

    let package_address = test_runner.publish_package("component");
    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_address,
            "CrossComponent",
            "create_component_with_auth",
            vec![scrypto_encode(&method_authorization)],
        )
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);
    receipt.result.expect("Should be okay");
    let secured_component = receipt.new_component_addresses[0];

    let transaction = test_runner
        .new_transaction_builder()
        .call_function(
            package_address,
            "CrossComponent",
            "create_component",
            vec![],
        )
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);
    assert!(receipt.result.is_ok());
    let my_component = receipt.new_component_addresses[0];

    let transaction = test_runner
        .new_transaction_builder()
        .withdraw_from_account_by_ids(&BTreeSet::from([auth_id.clone()]), auth, account)
        .call_method_with_all_resources(my_component, "put_auth")
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);
    assert!(receipt.result.is_ok());

    // Act
    let transaction = test_runner
        .new_transaction_builder()
        .call_method(
            my_component,
            "cross_component_call",
            vec![scrypto_encode(&secured_component)],
        )
        .build_and_sign(vec![], vec![])
        .unwrap();
    let receipt = test_runner.validate_and_execute(&transaction);

    // Assert
    assert!(receipt.result.is_ok());
}
