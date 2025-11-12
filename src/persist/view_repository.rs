use std::future::Future;

use crate::persist::PersistenceError;
use crate::{Aggregate, View};

/// Handles the database access needed for a GenericQuery.
pub trait ViewRepository<V, A>: Send + Sync
where
    V: View<A>,
    A: Aggregate,
{
    /// Returns the current view instance.
    fn load(
        &self,
        view_id: &str,
    ) -> impl Future<Output = Result<Option<V>, PersistenceError>> + Send;

    /// Returns the current view instance and context, used by the `GenericQuery` to update
    /// views with committed events.
    fn load_with_context(
        &self,
        view_id: &str,
    ) -> impl Future<Output = Result<Option<(V, ViewContext)>, PersistenceError>> + Send;

    /// Updates the view instance and context, used by the `GenericQuery` to update
    /// views with committed events.
    fn update_view(
        &self,
        view: V,
        context: ViewContext,
    ) -> impl Future<Output = Result<(), PersistenceError>> + Send;
}

/// A data structure maintaining context when updating views.
pub struct ViewContext {
    /// Unique identifier of the view instance that is being modified.
    pub view_instance_id: String,
    /// The current version of the view instance, used for optimistic locking.
    pub version: i64,
}

impl ViewContext {
    /// Convenience function to create a new [`ViewContext`].
    pub fn new(view_instance_id: String, version: i64) -> Self {
        Self {
            view_instance_id,
            version,
        }
    }
}
