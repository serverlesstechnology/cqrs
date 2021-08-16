use std::{
    collections::HashMap,
    sync::Arc,
};

use crate::{
    example_impl::*,
    memory_store::EventStore,
    AggregateError,
};

use super::repository::Repository;

type ThisEventStore =
    EventStore<CustomerCommand, CustomerEvent, Customer>;

fn metadata() -> HashMap<String, String> {
    let now = "2021-03-18T12:32:45.930Z".to_string();
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), now);
    metadata
}

#[test]
fn framework_test() {
    let event_store = ThisEventStore::default();
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
