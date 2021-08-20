use std::collections::HashMap;

use cqrs_es2_core::{
    example_impl::*,
    IEventHandler,
};

use crate::{
    memory_store::EventStore,
    IEventStore,
};

type ThisEventStore =
    EventStore<CustomerCommand, CustomerEvent, Customer>;

fn metadata() -> HashMap<String, String> {
    let now = "2021-03-18T12:32:45.930Z".to_string();
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), now);
    metadata
}

#[test]
fn test_memory_event_store() {
    let mut store = ThisEventStore::default();

    let id = "test_id_A";

    let initial_events = store.load_events(&id, true).unwrap();
    assert_eq!(0, initial_events.len());

    let agg_context = store.load_aggregate(&id).unwrap();

    store
        .commit(
            vec![CustomerEvent::NameAdded(NameAdded {
                changed_name: "test_event_A".to_string(),
            })],
            agg_context,
            metadata(),
        )
        .unwrap();

    let stored_events = store.load_events(&id, true).unwrap();
    assert_eq!(1, stored_events.len());

    let agg_context = store.load_aggregate(&id).unwrap();

    store
        .commit(
            vec![
                CustomerEvent::EmailUpdated(EmailUpdated {
                    new_email: "test A".to_string(),
                }),
                CustomerEvent::EmailUpdated(EmailUpdated {
                    new_email: "test B".to_string(),
                }),
                CustomerEvent::AddressUpdated(AddressUpdated {
                    new_address: "something else happening here"
                        .to_string(),
                }),
            ],
            agg_context,
            metadata(),
        )
        .unwrap();
    let stored_envelopes = store.load_events(&id, true).unwrap();

    let mut agg = Customer::default();
    for stored_envelope in stored_envelopes {
        let event = stored_envelope.payload;
        agg.apply(&event);
    }

    println!("{:#?}", agg);
}
