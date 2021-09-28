use async_trait::async_trait;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::persist::{EventStoreAggregateContext, PersistedEventRepository};
use crate::{Aggregate, AggregateError, EventEnvelope, EventStore};

/// Storage engine using a database backing.
/// This is an event-sourced `EventStore`, meaning it uses events as the
/// primary source of truth for the state of the aggregate.
///
/// For a snapshot-based `EventStore`
/// see [`PersistedSnapshotStore`](struct.PersistedSnapshotStore.html).
///
pub struct PersistedEventStore<R, A>
where
    R: PersistedEventRepository<A>,
    A: Aggregate + Send + Sync,
{
    repo: R,
    _phantom: PhantomData<A>,
}

impl<R, A> PersistedEventStore<R, A>
where
    R: PersistedEventRepository<A>,
    A: Aggregate + Send + Sync,
{
    /// Creates a new `PostgresStore` from the provided database connection,
    /// an `EventStore` used for configuring a new cqrs framework.
    ///
    /// ```ignore
    /// # use postgres_es::PostgresStore;
    /// # use cqrs_es::CqrsFramework;
    /// let store = PostgresStore::<MyAggregate>::new(pool);
    /// let cqrs = CqrsFramework::new(store, vec![]);
    /// ```
    pub fn new(repo: R) -> Self {
        PersistedEventStore {
            repo,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<R, A> EventStore<A> for PersistedEventStore<R, A>
where
    R: PersistedEventRepository<A>,
    A: Aggregate + Send + Sync,
{
    type AC = EventStoreAggregateContext<A>;

    async fn load(&self, aggregate_id: &str) -> Vec<EventEnvelope<A>> {
        match self.repo.get_events(aggregate_id).await {
            Ok(val) => val,
            Err(_err) => {
                // TODO: improved error handling
                Default::default()
            }
        }
    }
    async fn load_aggregate(&self, aggregate_id: &str) -> EventStoreAggregateContext<A> {
        let committed_events = self.load(aggregate_id).await;
        let mut aggregate = A::default();
        let mut current_sequence = 0;
        for envelope in committed_events {
            current_sequence = envelope.sequence;
            let event = envelope.payload;
            aggregate.apply(event);
        }
        EventStoreAggregateContext {
            aggregate_id: aggregate_id.to_string(),
            aggregate,
            current_sequence,
        }
    }

    async fn commit(
        &self,
        events: Vec<A::Event>,
        context: EventStoreAggregateContext<A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError> {
        let aggregate_id = context.aggregate_id.as_str();
        let current_sequence = context.current_sequence;
        let wrapped_events = self.wrap_events(aggregate_id, current_sequence, events, metadata);
        self.repo.insert_events(wrapped_events.clone()).await?;
        Ok(wrapped_events)
    }
}
