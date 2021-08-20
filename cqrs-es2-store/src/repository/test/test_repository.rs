use std::{
    collections::HashMap,
    sync::Arc,
};

use cqrs_es2_core::{
    example_impl::*,
    Error,
};

use crate::memory_store::{
    EventStore,
    QueryStore,
};

use crate::repository::Repository;

use super::dispatchers::CustomDispatcher;

type ThisEventStore =
    EventStore<CustomerCommand, CustomerEvent, Customer>;

type ThisQueryStore = QueryStore<
    CustomerCommand,
    CustomerEvent,
    Customer,
    CustomerContactQuery,
>;

fn metadata() -> HashMap<String, String> {
    let now = "2021-03-18T12:32:45.930Z".to_string();
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), now);
    metadata
}

#[test]
fn test_repository() {
    let event_store = ThisEventStore::default();
    let stored_events = event_store.get_events();

    let query_store = ThisQueryStore::default();

    let dispatched_events = Default::default();
    let custom_dispatcher =
        CustomDispatcher::new(Arc::clone(&dispatched_events));

    let mut repo = Repository::new(
        event_store,
        vec![
            Box::new(query_store),
            Box::new(custom_dispatcher),
        ],
    );

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
        dispatched_events.read().unwrap().len()
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
        dispatched_events.read().unwrap().len()
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
        Error::new(
            "this address has already been added for this customer"
        ),
        err
    );

    assert_eq!(
        2,
        dispatched_events.read().unwrap().len()
    );
    let stored_event_count = stored_events
        .read()
        .unwrap()
        .get(uuid.clone().as_str())
        .unwrap()
        .len();
    assert_eq!(2, stored_event_count);
}
