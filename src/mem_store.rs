use std::{
    collections::HashMap,
    sync::{
        Arc,
        RwLock,
    },
};

use crate::{
    aggregate::{
        Aggregate,
        AggregateError,
    },
    event::EventEnvelope,
    AggregateContext,
    EventStore,
};

///  Simple memory store only useful for testing purposes
pub struct MemStore<A: Aggregate> {
    events: Arc<LockedEventEnvelopeMap<A>>,
}

impl<A: Aggregate> Default for MemStore<A> {
    fn default() -> Self {
        let events = Default::default();
        MemStore { events }
    }
}

type LockedEventEnvelopeMap<A> =
    RwLock<HashMap<String, Vec<EventEnvelope<A>>>>;

impl<A: Aggregate> MemStore<A> {
    /// Get a shared copy of the events stored within the event store.
    pub fn get_events(&self) -> Arc<LockedEventEnvelopeMap<A>> {
        Arc::clone(&self.events)
    }
    fn load_commited_events(
        &self,
        aggregate_id: String,
    ) -> Vec<EventEnvelope<A>> {
        // uninteresting unwrap: this will not be used in production,
        // for tests only
        let event_map = self.events.read().unwrap();
        let mut committed_events: Vec<EventEnvelope<A>> = Vec::new();
        match event_map.get(aggregate_id.as_str()) {
            None => {},
            Some(events) => {
                for event in events {
                    committed_events.push(event.clone());
                }
            },
        };
        committed_events
    }
    fn aggregate_id(
        &self,
        events: &[EventEnvelope<A>],
    ) -> String {
        // uninteresting unwrap: this is not a struct for production
        // use
        let &first_event = events.iter().peekable().peek().unwrap();
        first_event.aggregate_id.to_string()
    }
}

impl<A: Aggregate> EventStore<A, MemStoreAggregateContext<A>>
    for MemStore<A>
{
    fn load(
        &self,
        aggregate_id: &str,
    ) -> Vec<EventEnvelope<A>> {
        let events =
            self.load_commited_events(aggregate_id.to_string());
        println!(
            "loading: {} events for aggregate ID '{}'",
            &events.len(),
            &aggregate_id
        );
        events
    }

    fn load_aggregate(
        &self,
        aggregate_id: &str,
    ) -> MemStoreAggregateContext<A> {
        let committed_events = self.load(aggregate_id);
        let mut aggregate = A::default();
        let mut current_sequence = 0;
        for envelope in committed_events {
            current_sequence = envelope.sequence;
            let event = envelope.payload;
            aggregate.apply(&event);
        }
        MemStoreAggregateContext {
            aggregate_id: aggregate_id.to_string(),
            aggregate,
            current_sequence,
        }
    }

    fn commit(
        &self,
        events: Vec<A::Event>,
        context: MemStoreAggregateContext<A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError> {
        let aggregate_id = context.aggregate_id.as_str();
        let current_sequence = context.current_sequence;
        let wrapped_events = self.wrap_events(
            aggregate_id,
            current_sequence,
            events,
            metadata,
        );
        let new_events_qty = wrapped_events.len();
        if new_events_qty == 0 {
            return Ok(Vec::default());
        }
        let aggregate_id = self.aggregate_id(&wrapped_events);
        let mut new_events =
            self.load_commited_events(aggregate_id.to_string());
        for event in &wrapped_events {
            new_events.push(event.clone());
        }
        println!(
            "storing: {} new events for aggregate ID '{}'",
            new_events_qty, &aggregate_id
        );
        // uninteresting unwrap: this is not a struct for production
        // use
        let mut event_map = self.events.write().unwrap();
        event_map.insert(aggregate_id, new_events);
        Ok(wrapped_events)
    }
}

/// Holds context for a pure event store implementation for MemStore
pub struct MemStoreAggregateContext<A>
where
    A: Aggregate, {
    /// The aggregate ID of the aggregate instance that has been
    /// loaded.
    pub aggregate_id: String,
    /// The current state of the aggregate instance.
    pub aggregate: A,
    /// The last committed event sequence number for this aggregate
    /// instance.
    pub current_sequence: usize,
}

impl<A> AggregateContext<A> for MemStoreAggregateContext<A>
where
    A: Aggregate,
{
    fn aggregate(&self) -> &A {
        &self.aggregate
    }
}
