use crate::persist::{PersistenceError, QueryContext, SerializedEvent, SerializedSnapshot};
use crate::{Aggregate, View};
use async_trait::async_trait;
use serde_json::Value;

/// Handles the database access needed for operation of a PersistedSnapshotStore.
#[async_trait]
pub trait PersistedEventRepository: Send + Sync {
    /// Returns all events for a single aggregate instance.
    async fn get_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<SerializedEvent>, PersistenceError>;

    /// Returns the last events for a single aggregate instance.
    async fn get_last_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
        number_events: usize,
    ) -> Result<Vec<SerializedEvent>, PersistenceError>;

    /// Returns the current snapshot for an aggregate instance.
    async fn get_snapshot<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<Option<SerializedSnapshot>, PersistenceError>;

    /// Commits the updated aggregate and accompanying events.
    async fn persist<A: Aggregate>(
        &self,
        events: &[SerializedEvent],
        snapshot_update: Option<(String, Value, usize)>,
    ) -> Result<(), PersistenceError>;
}

/// Handles the database access needed for a GenericQuery.
#[async_trait]
pub trait ViewRepository<V, A>: Send + Sync
where
    V: View<A>,
    A: Aggregate,
{
    /// Returns the current view instance.
    async fn load(
        &self,
        query_instance_id: &str,
    ) -> Result<Option<(V, QueryContext)>, PersistenceError>;

    /// Updates the view instance.
    async fn update_view(&self, view: V, context: QueryContext) -> Result<(), PersistenceError>;
}
