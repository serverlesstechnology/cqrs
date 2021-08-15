use std::{
    collections::HashMap,
    sync::{
        Arc,
        RwLock,
    },
};

use crate::{
    aggregates::{
        AggregateContext,
        IAggregate,
    },
    errors::AggregateError,
    events::EventContext,
};

use super::super::IEventStore;

///  Simple memory store only useful for testing purposes
pub struct EventStore<A: IAggregate> {
    events: Arc<LockedEventEnvelopeMap<A>>,
}

impl<A: IAggregate> Default for EventStore<A> {
    fn default() -> Self {
        let events = Default::default();
        EventStore { events }
    }
}

type LockedEventEnvelopeMap<A> =
    RwLock<HashMap<String, Vec<EventContext<A>>>>;

impl<A: IAggregate> EventStore<A> {
    /// Get a shared copy of the events stored within the event store.
    pub fn get_events(&self) -> Arc<LockedEventEnvelopeMap<A>> {
        Arc::clone(&self.events)
    }

    fn load_committed_events(
        &self,
        aggregate_id: String,
    ) -> Vec<EventContext<A>> {
        // uninteresting unwrap: this will not be used in production,
        // for tests only
        let event_map = self.events.read().unwrap();
        let mut committed_events: Vec<EventContext<A>> = Vec::new();

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
        events: &[EventContext<A>],
    ) -> String {
        // uninteresting unwrap: this is not a struct for production
        // use
        let &first_event = events.iter().peekable().peek().unwrap();
        first_event.aggregate_id.to_string()
    }
}

impl<A: IAggregate> IEventStore<A> for EventStore<A> {
    fn load_events(
        &mut self,
        aggregate_id: &str,
        with_metadata: bool,
    ) -> Result<Vec<EventContext<A>>, AggregateError> {
        let event_contexts =
            self.load_committed_events(aggregate_id.to_string());

        if with_metadata {
            return Ok(event_contexts);
        }

        // clear the metadata to simulate loading events only
        let mut events = Vec::new();

        for event_context in event_contexts {
            events.push(EventContext {
                aggregate_id: event_context.aggregate_id,
                sequence: event_context.sequence,
                payload: event_context.payload,
                metadata: Default::default(),
            });
        }

        println!(
            "loading: {} events for aggregate ID '{}'",
            &events.len(),
            &aggregate_id
        );

        Ok(events)
    }

    fn load_aggregate(
        &mut self,
        aggregate_id: &str,
    ) -> Result<AggregateContext<A>, AggregateError> {
        let committed_events =
            match self.load_events(aggregate_id, false) {
                Ok(x) => x,
                Err(e) => {
                    return Err(e);
                },
            };

        let mut aggregate = A::default();
        let mut current_sequence = 0;

        for envelope in committed_events {
            current_sequence = envelope.sequence;
            let event = envelope.payload;
            aggregate.apply(&event);
        }

        Ok(AggregateContext {
            aggregate_id: aggregate_id.to_string(),
            aggregate,
            current_sequence,
        })
    }

    fn commit(
        &mut self,
        events: Vec<A::Event>,
        context: AggregateContext<A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventContext<A>>, AggregateError> {
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
            self.load_committed_events(aggregate_id.to_string());

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
