use crate::example_impl::*;

use super::test_framework::TestFramework;

type ThisTestFramework =
    TestFramework<CustomerCommand, CustomerEvent, Customer>;

#[test]
fn test_framework_test() {
    let test_name = "test A";
    let test_framework = ThisTestFramework::default();

    test_framework
        .given(vec![CustomerEvent::NameAdded(
            NameAdded {
                changed_name: "test_id_A".to_string(),
            },
        )])
        .when(CustomerCommand::UpdateEmail(
            UpdateEmail {
                new_email: test_name.to_string(),
            },
        ))
        .then_expect_events(vec![CustomerEvent::EmailUpdated(
            EmailUpdated {
                new_email: test_name.to_string(),
            },
        )]);

    test_framework
        .given(vec![CustomerEvent::NameAdded(
            NameAdded {
                changed_name: test_name.to_string(),
            },
        )])
        .when(CustomerCommand::AddCustomerName(
            AddCustomerName {
                changed_name: test_name.to_string(),
            },
        ))
        .then_expect_error(
            "a name has already been added for this customer",
        )
}

#[test]
#[should_panic]
fn test_framework_failure_test() {
    let test_name = "test A";
    let test_framework = ThisTestFramework::default();

    test_framework
        .given(vec![CustomerEvent::NameAdded(
            NameAdded {
                changed_name: test_name.to_string(),
            },
        )])
        .when(CustomerCommand::AddCustomerName(
            AddCustomerName {
                changed_name: test_name.to_string(),
            },
        ))
        .then_expect_events(vec![CustomerEvent::NameAdded(
            NameAdded {
                changed_name: test_name.to_string(),
            },
        )]);
}

#[test]
#[should_panic]
fn test_framework_failure_test_b() {
    let test_name = "test A";
    let test_framework = ThisTestFramework::default();

    test_framework
        .given(vec![CustomerEvent::NameAdded(
            NameAdded {
                changed_name: "test_id_A".to_string(),
            },
        )])
        .when(CustomerCommand::UpdateEmail(
            UpdateEmail {
                new_email: test_name.to_string(),
            },
        ))
        .then_expect_error("some error message")
}
