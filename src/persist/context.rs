use crate::{Aggregate, AggregateContext};

/// Holds context for the pure event store implementation PostgresStore.
/// This is only used internally within the `EventStore`.
pub struct EventStoreAggregateContext<A: Aggregate> {
    /// The aggregate ID of the aggregate instance that has been loaded.
    pub aggregate_id: String,
    /// The current state of the aggregate instance.
    pub aggregate: A,
    /// The last committed event sequence number for this aggregate instance.
    pub current_sequence: usize,
}

impl<A: Aggregate> SnapshotStoreAggregateContext<A> {
    /// Creates a new, default context with the given aggregate id.
    pub fn new(aggregate_id: &str) -> Self {
        Self {
            aggregate_id: aggregate_id.to_string(),
            aggregate: Default::default(),
            current_sequence: 0,
            current_snapshot: 0,
        }
    }
}

impl<A: Aggregate> AggregateContext<A> for EventStoreAggregateContext<A> {
    fn aggregate(&self) -> &A {
        &self.aggregate
    }
}

/// Holds context for the snapshot-sourced implementation PostgresSnapshotStore.
/// This is only used internally within the `EventStore`.
#[derive(Debug, PartialEq)]
pub struct SnapshotStoreAggregateContext<A>
where
    A: Aggregate,
{
    /// The aggregate ID of the aggregate instance that has been loaded.
    pub aggregate_id: String,
    /// The current state of the aggregate instance.
    pub aggregate: A,
    /// The last committed event sequence number for this aggregate instance.
    pub current_sequence: usize,
    /// The last committed snapshot version for this aggregate instance.
    pub current_snapshot: usize,
}

impl<A> AggregateContext<A> for SnapshotStoreAggregateContext<A>
where
    A: Aggregate,
{
    fn aggregate(&self) -> &A {
        &self.aggregate
    }
}

/// A data structure maintaining context when updating views.
pub struct QueryContext {
    /// Unique identifier of the view instance that is being modified.
    pub view_instance_id: String,
    /// The current version of the view instance, used for optimistic locking.
    pub version: i64,
}

impl QueryContext {
    /// Convenience function to create a new QueryContext.
    pub fn new(view_instance_id: String, version: i64) -> Self {
        Self {
            view_instance_id,
            version,
        }
    }
}
