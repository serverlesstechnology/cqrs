use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use crate::event::EventEnvelope;
use crate::{Aggregate, AggregateContext, AggregateError, EventStore};

///  Simple memory store useful for application development and testing purposes.
///
/// Creation and use in a constructing a `CqrsFramework`:
/// ```
/// # use cqrs_es::doc::{MyAggregate, MyService};
/// use cqrs_es::CqrsFramework;
/// use cqrs_es::mem_store::MemStore;
///
/// let store = MemStore::<MyAggregate>::default();
/// let cqrs = CqrsFramework::new(store, vec![], MyService);
/// ```
pub struct MemStore<A: Aggregate + Send + Sync> {
    events: Arc<LockedEventEnvelopeMap<A>>,
}

impl<A: Aggregate> Default for MemStore<A> {
    fn default() -> Self {
        let events = Arc::default();
        Self { events }
    }
}

type LockedEventEnvelopeMap<A> = RwLock<HashMap<String, Vec<EventEnvelope<A>>>>;

impl<A: Aggregate> MemStore<A> {
    /// Get a shared copy of the events stored within the event store.
    ///
    /// This can be used to verify the state of events that have been committed.
    /// Example of reading and displaying stored events:
    /// ```
    /// # use cqrs_es::doc::MyAggregate;
    /// # use cqrs_es::EventEnvelope;
    /// # use cqrs_es::mem_store::MemStore;
    /// let store = MemStore::<MyAggregate>::default();
    /// //...
    /// let all_locked_events = store.get_events();
    /// let unlocked_events = all_locked_events.read().unwrap();
    /// match unlocked_events.get("test-aggregate-id-C450D1A") {
    ///     Some(events) => {
    ///         for event in events {
    ///             println!("{:?}", event);
    ///         }
    ///     }
    ///     None => {}
    /// };
    /// ```
    pub fn get_events(&self) -> Arc<LockedEventEnvelopeMap<A>> {
        Arc::clone(&self.events)
    }

    fn load_commited_events(
        &self,
        aggregate_id: String,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError<A::Error>> {
        // uninteresting unwrap: this will not be used in production, for tests only
        let event_map = self.events.read().unwrap();
        let mut committed_events: Vec<EventEnvelope<A>> = Vec::new();
        match event_map.get(aggregate_id.as_str()) {
            None => {}
            Some(events) => {
                for event in events {
                    committed_events.push(event.clone());
                }
            }
        };
        Ok(committed_events)
    }

    fn aggregate_id(&self, events: &[EventEnvelope<A>]) -> String {
        // uninteresting unwrap: this is not a struct for production use
        let &first_event = events.iter().peekable().peek().unwrap();
        first_event.aggregate_id.to_string()
    }
}

#[async_trait]
impl<A: Aggregate> EventStore<A> for MemStore<A> {
    type AC = MemStoreAggregateContext<A>;

    async fn load_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError<A::Error>> {
        let events = self.load_commited_events(aggregate_id.to_string())?;
        println!(
            "loading: {} events for aggregate ID '{}'",
            &events.len(),
            &aggregate_id
        );
        Ok(events)
    }

    async fn load_aggregate(
        &self,
        aggregate_id: &str,
    ) -> Result<MemStoreAggregateContext<A>, AggregateError<A::Error>> {
        let committed_events = self.load_events(aggregate_id).await?;
        let mut aggregate = A::default();
        let mut current_sequence = 0;
        for envelope in committed_events {
            current_sequence = envelope.sequence;
            let event = envelope.payload;
            aggregate.apply(event);
        }
        Ok(MemStoreAggregateContext {
            aggregate_id: aggregate_id.to_string(),
            aggregate,
            current_sequence,
        })
    }

    async fn commit(
        &self,
        events: Vec<A::Event>,
        context: MemStoreAggregateContext<A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError<A::Error>> {
        let aggregate_id = context.aggregate_id.as_str();
        let current_sequence = context.current_sequence;
        let wrapped_events = self.wrap_events(aggregate_id, current_sequence, events, metadata);
        let new_events_qty = wrapped_events.len();
        if new_events_qty == 0 {
            return Ok(Vec::default());
        }
        let aggregate_id = self.aggregate_id(&wrapped_events);
        let mut new_events = self.load_commited_events(aggregate_id.to_string()).unwrap();
        for event in &wrapped_events {
            new_events.push(event.clone());
        }
        println!(
            "storing: {} new events for aggregate ID '{}'",
            new_events_qty, &aggregate_id
        );
        // uninteresting unwrap: this is not a struct for production use
        let mut event_map = self.events.write().unwrap();
        event_map.insert(aggregate_id, new_events);
        Ok(wrapped_events)
    }
}

impl<A: Aggregate> MemStore<A> {
    /// Method to wrap a set of events with the additional metadata needed for persistence and publishing
    fn wrap_events(
        &self,
        aggregate_id: &str,
        current_sequence: usize,
        resultant_events: Vec<A::Event>,
        base_metadata: HashMap<String, String>,
    ) -> Vec<EventEnvelope<A>> {
        let mut sequence = current_sequence;
        let mut wrapped_events: Vec<EventEnvelope<A>> = Vec::new();
        for payload in resultant_events {
            sequence += 1;
            let aggregate_id: String = aggregate_id.to_string();
            let sequence = sequence;
            let metadata = base_metadata.clone();
            wrapped_events.push(EventEnvelope {
                aggregate_id,
                sequence,
                payload,
                metadata,
            });
        }
        wrapped_events
    }
}
/// Holds context for a pure event store implementation for MemStore.
///
/// This is used internally by the `CqrsFramework`.
pub struct MemStoreAggregateContext<A>
where
    A: Aggregate,
{
    /// The aggregate ID of the aggregate instance that has been loaded.
    pub aggregate_id: String,
    /// The current state of the aggregate instance.
    pub aggregate: A,
    /// The last committed event sequence number for this aggregate instance.
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
