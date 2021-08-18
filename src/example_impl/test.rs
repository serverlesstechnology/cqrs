use crate::TestFramework;

use super::{
    aggregate::Customer,
    commands::*,
    events::*,
};

type ThisTestFramework =
    TestFramework<CustomerCommand, CustomerEvent, Customer>;

#[test]
fn test_change_name() {
    ThisTestFramework::default()
        .given_no_previous_events()
        .when(CustomerCommand::AddCustomerName(
            AddCustomerName {
                changed_name: "John Doe".to_string(),
            },
        ))
        .then_expect_events(vec![CustomerEvent::NameAdded(
            NameAdded {
                changed_name: "John Doe".to_string(),
            },
        )]);
}

#[test]
fn test_change_name_again() {
    ThisTestFramework::default()
        .given(vec![CustomerEvent::NameAdded(
            NameAdded {
                changed_name: "John Doe".to_string(),
            },
        )])
        .when(CustomerCommand::AddCustomerName(
            AddCustomerName {
                changed_name: "John Doe".to_string(),
            },
        ))
        .then_expect_error(
            "a name has already been added for this customer",
        );
}
