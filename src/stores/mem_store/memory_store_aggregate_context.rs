use crate::aggregates::Aggregate;

use super::super::AggregateContext;

/// Holds context for a pure event store implementation for
/// MemoryStore
pub struct MemoryStoreAggregateContext<A>
where
    A: Aggregate, {
    /// The aggregate ID of the aggregate instance that has been
    /// loaded.
    pub aggregate_id: String,
    /// The current state of the aggregate instance.
    pub aggregate: A,
    /// The last committed event sequence number for this aggregate
    /// instance.
    pub current_sequence: usize,
}

impl<A> AggregateContext<A> for MemoryStoreAggregateContext<A>
where
    A: Aggregate,
{
    fn aggregate(&self) -> &A {
        &self.aggregate
    }
}
