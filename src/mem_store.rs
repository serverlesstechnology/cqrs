use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::aggregate::{Aggregate, AggregateError};
use crate::event::{DomainEvent, EventEnvelope};
use crate::EventStore;

///  Simple memory store only useful for testing purposes
pub struct MemStore<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    events: Arc<LockedEventEnvelopeMap<A, E>>,
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

type LockedEventEnvelopeMap<A, E> = RwLock<HashMap<String, Vec<EventEnvelope<A, E>>>>;

impl<A, E> MemStore<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    /// Get a shared copy of the events stored within the event store.
    pub fn get_events(&self) -> Arc<LockedEventEnvelopeMap<A, E>> {
        Arc::clone(&self.events)
    }
    fn load_commited_events(&self, aggregate_id: String) -> Vec<EventEnvelope<A, E>> {
        // uninteresting unwrap: this will not be used in production, for tests only
        let event_map = self.events.read().unwrap();
        let mut committed_events: Vec<EventEnvelope<A, E>> = Vec::new();
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
    fn aggregate_id(&self, events: &[EventEnvelope<A, E>]) -> String {
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
    fn load(&self, aggregate_id: &str) -> Vec<EventEnvelope<A, E>>
    {
        let events = self.load_commited_events(aggregate_id.to_string());
        println!("loading: {} events for aggregate ID '{}'", &events.len(), &aggregate_id);
        events
    }

    fn commit(&self, events: Vec<EventEnvelope<A, E>>) -> Result<(), AggregateError> {
        let aggregate_id = self.aggregate_id(&events);
        let mut new_events = self.load_commited_events(aggregate_id.to_string());
        for event in events {
            new_events.push(event.clone());
        }
        println!("storing: {} events for aggregate ID '{}'", &new_events.len(), &aggregate_id);
        // uninteresting unwrap: this is not a struct for production use
        let mut event_map = self.events.write().unwrap();
        event_map.insert(aggregate_id, new_events);
        Ok(())
    }
}