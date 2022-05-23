use crate::persist::PersistenceError;
use crate::{Aggregate, View};
use async_trait::async_trait;

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

/// A data structure maintaining context when updating views.
pub struct ViewContext {
    /// Unique identifier of the view instance that is being modified.
    pub view_instance_id: String,
    /// The current version of the view instance, used for optimistic locking.
    pub version: i64,
}

impl ViewContext {
    /// Convenience function to create a new QueryContext.
    pub fn new(view_instance_id: String, version: i64) -> Self {
        Self {
            view_instance_id,
            version,
        }
    }
}
