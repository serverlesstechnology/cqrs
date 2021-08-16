use std::{
    collections::HashMap,
    sync::Arc,
};

use cqrs_es2::{
    example_impl::*,
    memory_store::EventStore as MemoryEventStore,
    AggregateError,
    Repository,
    TestFramework,
};

type ThisTestFramework = TestFramework<Customer>;

fn metadata() -> HashMap<String, String> {
    let now = "2021-03-18T12:32:45.930Z".to_string();
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), now);
    metadata
}

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

#[test]
fn framework_test() {
    let event_store = MemoryEventStore::default();
    let stored_events = event_store.get_events();

    let delivered_events = Default::default();
    let view = TestView::new(Arc::clone(&delivered_events));

    let mut repo = Repository::new(event_store, vec![Box::new(view)]);
    let uuid = uuid::Uuid::new_v4().to_string();
    let id = uuid.clone();
    let metadata = metadata();
    repo.execute_with_metadata(
        &id,
        CustomerCommand::AddAddress(AddAddress {
            new_address: uuid.clone(),
        }),
        metadata,
    )
    .unwrap_or_default();

    assert_eq!(1, stored_events.read().unwrap().len());
    assert_eq!(
        1,
        delivered_events.read().unwrap().len()
    );

    let test = "TEST_A";
    let id = uuid.clone();
    repo.execute(
        &id,
        CustomerCommand::AddAddress(AddAddress {
            new_address: test.to_string(),
        }),
    )
    .unwrap_or_default();

    assert_eq!(
        2,
        delivered_events.read().unwrap().len()
    );
    let stored_event_count = stored_events
        .read()
        .unwrap()
        .get(uuid.clone().as_str())
        .unwrap()
        .len();
    assert_eq!(2, stored_event_count);

    let id = uuid.clone();
    let err = repo
        .execute(
            &id,
            CustomerCommand::AddAddress(AddAddress {
                new_address: test.to_string(),
            }),
        )
        .unwrap_err();
    assert_eq!(
        AggregateError::new(
            "this address has already been added for this customer"
        ),
        err
    );

    assert_eq!(
        2,
        delivered_events.read().unwrap().len()
    );
    let stored_event_count = stored_events
        .read()
        .unwrap()
        .get(uuid.clone().as_str())
        .unwrap()
        .len();
    assert_eq!(2, stored_event_count);
}
