use std::collections::HashMap;

use crate::{
    example_impl::*,
    IEventHandler,
    IEventStore,
};

use super::event_store::EventStore;

type ThisEventStore =
    EventStore<CustomerCommand, CustomerEvent, Customer>;

fn metadata() -> HashMap<String, String> {
    let now = "2021-03-18T12:32:45.930Z".to_string();
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), now);
    metadata
}

#[test]
fn test_mem_store() {
    let mut event_store = ThisEventStore::default();
    let id = "test_id_A";
    let initial_events = event_store
        .load_events(&id, true)
        .unwrap();
    assert_eq!(0, initial_events.len());
    let agg_context = event_store.load_aggregate(&id).unwrap();

    event_store
        .commit(
            vec![CustomerEvent::NameAdded(NameAdded {
                changed_name: "test_event_A".to_string(),
            })],
            agg_context,
            metadata(),
        )
        .unwrap();
    let stored_events = event_store
        .load_events(&id, true)
        .unwrap();
    assert_eq!(1, stored_events.len());
    let agg_context = event_store.load_aggregate(&id).unwrap();

    event_store
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
    let stored_envelopes = event_store
        .load_events(&id, true)
        .unwrap();

    let mut agg = Customer::default();
    for stored_envelope in stored_envelopes {
        let event = stored_envelope.payload;
        agg.apply(&event);
    }

    println!("{:#?}", agg);
}
