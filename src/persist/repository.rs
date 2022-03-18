use crate::persist::{PersistenceError, SerializedEvent, SerializedSnapshot, ViewContext};
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
        last_sequence: usize,
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
    async fn load(&self, view_id: &str) -> Result<Option<V>, PersistenceError>;

    /// Returns the current view instance and context, used by the `GenericQuery` to update
    /// views with committed events.
    async fn load_with_context(
        &self,
        view_id: &str,
    ) -> Result<Option<(V, ViewContext)>, PersistenceError>;

    /// Updates the view instance and context, used by the `GenericQuery` to update
    /// views with committed events.
    async fn update_view(&self, view: V, context: ViewContext) -> Result<(), PersistenceError>;
}
