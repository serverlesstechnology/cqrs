use std::collections::HashMap;

use postgres::{
    Client,
    NoTls,
};

use cqrs_es2_core::{
    example_impl::*,
    AggregateContext,
    EventContext,
};

use crate::{
    postgres_store::EventStore,
    IEventStore,
};

use super::common::*;

type ThisEventStore =
    EventStore<CustomerCommand, CustomerEvent, Customer>;

type ThisAggregateContext =
    AggregateContext<CustomerCommand, CustomerEvent, Customer>;

type ThisEventContext = EventContext<CustomerCommand, CustomerEvent>;

pub fn metadata() -> HashMap<String, String> {
    let now = "2021-03-18T12:32:45.930Z".to_string();
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), now);
    metadata
}

fn commit_and_load_events(with_snapshots: bool) {
    let conn = Client::connect(CONNECTION_STRING, NoTls).unwrap();
    let mut store = ThisEventStore::new(conn, with_snapshots);

    let id = uuid::Uuid::new_v4().to_string();

    // loading nonexisting stream defaults to a vector with zero
    // length
    assert_eq!(
        0,
        store
            .load_events(id.as_str(), false)
            .unwrap()
            .len()
    );

    // loading nonexisting aggregate returns default construction
    let context = store
        .load_aggregate(id.as_str())
        .unwrap();

    assert_eq!(
        context,
        ThisAggregateContext::new(id.clone(), Customer::default(), 0)
    );

    // apply a couple of events
    let events = vec![
        CustomerEvent::NameAdded(NameAdded {
            changed_name: "test_event_A".to_string(),
        }),
        CustomerEvent::EmailUpdated(EmailUpdated {
            new_email: "test A".to_string(),
        }),
        CustomerEvent::AddressUpdated(AddressUpdated {
            new_address: "test B".to_string(),
        }),
    ];

    store
        .commit(
            vec![events[0].clone(), events[1].clone()],
            context,
            metadata(),
        )
        .unwrap();

    let contexts = store
        .load_events(id.as_str(), false)
        .unwrap();

    // check stored events are correct
    assert_eq!(
        contexts,
        vec![
            ThisEventContext::new(
                id.to_string(),
                1,
                events[0].clone(),
                Default::default()
            ),
            ThisEventContext::new(
                id.to_string(),
                2,
                events[1].clone(),
                Default::default()
            ),
        ]
    );

    let context = store
        .load_aggregate(id.as_str())
        .unwrap();

    // check stored aggregate is correct
    assert_eq!(
        context,
        ThisAggregateContext::new(
            id.clone(),
            Customer {
                customer_id: "".to_string(),
                name: "test_event_A".to_string(),
                email: "test A".to_string(),
                addresses: Default::default()
            },
            2
        )
    );

    store
        .commit(
            vec![events[2].clone()],
            context,
            metadata(),
        )
        .unwrap();

    let contexts = store
        .load_events(id.as_str(), false)
        .unwrap();

    // check stored events are correct
    assert_eq!(
        contexts,
        vec![
            ThisEventContext::new(
                id.to_string(),
                1,
                events[0].clone(),
                Default::default()
            ),
            ThisEventContext::new(
                id.to_string(),
                2,
                events[1].clone(),
                Default::default()
            ),
            ThisEventContext::new(
                id.to_string(),
                3,
                events[2].clone(),
                Default::default()
            ),
        ]
    );

    let context = store
        .load_aggregate(id.as_str())
        .unwrap();

    // check stored aggregate is correct
    assert_eq!(
        context,
        ThisAggregateContext::new(
            id.clone(),
            Customer {
                customer_id: "".to_string(),
                name: "test_event_A".to_string(),
                email: "test A".to_string(),
                addresses: vec!["test B".to_string()]
            },
            3
        )
    );
}

#[test]
fn test_with_snapshots() {
    commit_and_load_events(true);
}

#[test]
fn test_no_snapshots() {
    commit_and_load_events(false);
}
