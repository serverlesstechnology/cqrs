use crate::persist::error::PersistenceError;
use crate::persist::SnapshotStoreAggregateContext;
use crate::{Aggregate, EventEnvelope};
use async_trait::async_trait;

#[async_trait]
pub trait PersistedEventRepository<A>: Send + Sync
where
    A: Aggregate,
{
    async fn get_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<EventEnvelope<A>>, PersistenceError>;

    async fn insert_events(&self, events: Vec<EventEnvelope<A>>) -> Result<(), PersistenceError>;
}

#[async_trait]
pub trait PersistedSnapshotEventRepository<A>: Send + Sync
where
    A: Aggregate,
{
    async fn get_snapshot(
        &self,
        aggregate_id: &str,
    ) -> Result<Option<SnapshotStoreAggregateContext<A>>, PersistenceError>;

    async fn persist(
        &self,
        aggregate: A,
        aggregate_id: String,
        current_snapshot: usize,
        events: &[EventEnvelope<A>],
    ) -> Result<(), PersistenceError>;
}
