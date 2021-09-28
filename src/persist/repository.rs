use crate::persist::error::PersistenceError;
use crate::persist::SnapshotStoreAggregateContext;
use crate::{Aggregate, EventEnvelope};
use async_trait::async_trait;

/// Handles the database access needed for operation of a PersistedEventStore.
#[async_trait]
pub trait PersistedEventRepository<A>: Send + Sync
where
    A: Aggregate,
{
    /// Returns all events for a single aggregate instance.
    async fn get_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<EventEnvelope<A>>, PersistenceError>;

    /// Commits new events into the database.
    async fn insert_events(&self, events: Vec<EventEnvelope<A>>) -> Result<(), PersistenceError>;
}

/// Handles the database access needed for operation of a PersistedSnapshotStore.
#[async_trait]
pub trait PersistedSnapshotEventRepository<A>: Send + Sync
where
    A: Aggregate,
{
    /// Returns the current snapshot for an aggregate instance.
    async fn get_snapshot(
        &self,
        aggregate_id: &str,
    ) -> Result<Option<SnapshotStoreAggregateContext<A>>, PersistenceError>;

    /// Commits the updated aggregate and accompanying events.
    async fn persist(
        &self,
        aggregate: A,
        aggregate_id: String,
        current_snapshot: usize,
        events: &[EventEnvelope<A>],
    ) -> Result<(), PersistenceError>;
}
