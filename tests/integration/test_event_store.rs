use std::{
    rc::Rc,
    sync::RwLock,
};

use postgres::{
    Client,
    NoTls,
};
use serde_json::{
    Map,
    Value,
};

use cqrs_es2::{
    example_impl::*,
    postgres_store::EventStore,
    EventContext,
    IAggregate,
    IEventStore,
};

use super::common_one::*;

fn test_store() -> EventStore<Customer> {
    let conn = Client::connect(CONNECTION_STRING, NoTls).unwrap();
    EventStore::<Customer>::new(conn, true)
}

// #[test]
// fn test_valid_cqrs_framework() {
//     let view_events: Rc<RwLock<Vec<EventContext<Customer>>>> =
//         Default::default();
//     let query = TestQuery::new(view_events);
//     let conn = Client::connect(CONNECTION_STRING, NoTls).unwrap();
//     let _ps = get_cqrs(conn, true, vec![Box::new(query)]);
// }

#[test]
fn commit_and_load_events() {
    let mut event_store = test_store();
    let id = uuid::Uuid::new_v4().to_string();
    assert_eq!(
        0,
        event_store
            .load_events(id.as_str(), false)
            .unwrap()
            .len()
    );
    let context = event_store
        .load_aggregate(id.as_str())
        .unwrap();

    event_store
        .commit(
            vec![
                CustomerEvent::NameAdded(NameAdded {
                    changed_name: "test_event_A".to_string(),
                }),
                CustomerEvent::EmailUpdated(EmailUpdated {
                    new_email: "test A".to_string(),
                }),
            ],
            context,
            metadata(),
        )
        .unwrap();

    assert_eq!(
        2,
        event_store
            .load_events(id.as_str(), true)
            .unwrap()
            .len()
    );
    let context = event_store
        .load_aggregate(id.as_str())
        .unwrap();

    event_store
        .commit(
            vec![CustomerEvent::EmailUpdated(
                EmailUpdated {
                    new_email: "test B".to_string(),
                },
            )],
            context,
            metadata(),
        )
        .unwrap();

    assert_eq!(
        3,
        event_store
            .load_events(id.as_str(), true)
            .unwrap()
            .len()
    );
}

#[test]
fn test_event_breakout_type() {
    let event = CustomerEvent::NameAdded(NameAdded {
        changed_name: "test_event_A".to_string(),
    });

    let (event_type, value) = serialize_event::<Customer>(&event);
    println!("{} - {}", &event_type, &value);
    let test_event: CustomerEvent =
        deserialize_event::<Customer>(event_type.as_str(), value);
    assert_eq!(test_event, event);
}

fn serialize_event<A: IAggregate>(
    event: &A::Event
) -> (String, Value) {
    let val = serde_json::to_value(event).unwrap();
    match &val {
        Value::Object(object) => {
            for key in object.keys() {
                let value = object.get(key).unwrap();
                return (key.to_string(), value.clone());
            }
            panic!("{:?} not a domain event", val);
        },
        _ => {
            panic!("{:?} not an object", val);
        },
    }
}

fn deserialize_event<A: IAggregate>(
    event_type: &str,
    value: Value,
) -> A::Event {
    let mut new_val_map = Map::with_capacity(1);
    new_val_map.insert(event_type.to_string(), value);
    let new_event_val = Value::Object(new_val_map);
    serde_json::from_value(new_event_val).unwrap()
}

#[test]
fn thread_safe_test() {
    // TODO: use R2D2 for sync/send
    // https://github.com/sfackler/r2d2-postgres
    // fn is_sync<T: Sync>() {}
    // is_sync::<EventStore<Customer>>();
    fn is_send<T: Send>() {}
    is_send::<EventStore<Customer>>();
}

// #[test]
// fn commit_and_load_events_snapshot_store() {
//     let mut event_store = test_snapshot_store();
//     let id = uuid::Uuid::new_v4().to_string();
//     assert_eq!(
//         0,
//         event_store
//             .load_events(id.as_str())
//             .len()
//     );
//     let context = event_store.load_aggregate(id.as_str());

//     event_store
//         .commit(
//             vec![
//                 TestEvent::Created(Created {
//                     id: "test_event_A".to_string(),
//                 }),
//                 TestEvent::Tested(Tested {
//                     test_name: "test A".to_string(),
//                 }),
//             ],
//             context,
//             metadata(),
//         )
//         .unwrap();

//     assert_eq!(
//         2,
//         event_store
//             .load_events(id.as_str())
//             .len()
//     );
//     let context = event_store.load_aggregate(id.as_str());

//     event_store
//         .commit(
//             vec![TestEvent::Tested(Tested {
//                 test_name: "test B".to_string(),
//             })],
//             context,
//             metadata(),
//         )
//         .unwrap();
//     assert_eq!(
//         3,
//         event_store
//             .load_events(id.as_str())
//             .len()
//     );
// }
