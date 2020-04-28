use std::collections::HashMap;
use std::rc::Rc;
use std::sync::RwLock;

use crate::aggregate::{Aggregate, AggregateError};
use crate::event::{DomainEvent, MessageEnvelope};
use crate::EventStore;


///  Simple memory store only useful for testing purposes
pub struct MemStore<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    events: Rc<LockedMessageEnvelopeMap<A, E>>,
}

impl<A, E> Default for MemStore<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    fn default() -> Self {
        let events = Default::default();
        MemStore {
            events,
        }
    }
}

type LockedMessageEnvelopeMap<A, E> = RwLock<HashMap<String, Vec<MessageEnvelope<A, E>>>>;

impl<A, E> MemStore<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    /// Get a shared copy of the events stored within the event store.
    pub fn get_events(&self) -> Rc<LockedMessageEnvelopeMap<A, E>> {
        Rc::clone(&self.events)
    }
    fn load_commited_events(&self, aggregate_id: String) -> Vec<MessageEnvelope<A, E>> {
        // uninteresting unwrap: this will not be used in production, for tests only
        let event_map = self.events.read().unwrap();
        let mut committed_events: Vec<MessageEnvelope<A, E>> = Vec::new();
        match event_map.get(aggregate_id.as_str()) {
            None => {}
            Some(events) => {
                for event in events {
                    committed_events.push(event.clone());
                }
            }
        };
        committed_events
    }
    fn aggregate_id(&self, events: &[MessageEnvelope<A, E>]) -> String {
        // uninteresting unwrap: this is not a struct for production use
        let &first_event = events.iter().peekable().peek().unwrap();
        first_event.aggregate_id.to_string()
    }
}

impl<A, E> EventStore<A, E> for MemStore<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    fn load(&self, aggregate_id: &str) -> Vec<MessageEnvelope<A, E>>
    {
        let events = self.load_commited_events(aggregate_id.to_string());
        println!("loading: {} events", &events.len());
        events
    }

    fn commit(&self, events: Vec<MessageEnvelope<A, E>>) -> Result<(), AggregateError> {
        let aggregate_id = self.aggregate_id(&events);
        let mut new_events = self.load_commited_events(aggregate_id.to_string());
        for event in events {
            new_events.push(event.clone());
        }
        println!("storing: {} events", &new_events.len());
        // uninteresting unwrap: this is not a struct for production use
        let mut event_map = self.events.write().unwrap();
        event_map.insert(aggregate_id, new_events);
        Ok(())
    }
}