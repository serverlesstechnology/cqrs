use crate::persist::event_stream::ReplayStream;
use crate::persist::{PersistenceError, SerializedEvent, SerializedSnapshot};
use crate::Aggregate;
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

    /// Streams all events for an aggregate instance.
    async fn stream_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<ReplayStream, PersistenceError>;

    /// Streams all events for an aggregate type.
    async fn stream_all_events<A: Aggregate>(&self) -> Result<ReplayStream, PersistenceError>;
}
