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
