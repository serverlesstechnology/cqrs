use crate::aggregates::{
    IAggregate,
    IAggregateContext,
};

/// Holds context for a pure event store implementation for
/// MemoryStore
pub struct AggregateContext<A: IAggregate> {
    /// The aggregate ID of the aggregate instance that has been
    /// loaded.
    pub aggregate_id: String,

    /// The current state of the aggregate instance.
    pub aggregate: A,

    /// The last committed event sequence number for this aggregate
    /// instance.
    pub current_sequence: usize,
}

impl<A: IAggregate> IAggregateContext<A> for AggregateContext<A> {
    fn aggregate(&self) -> &A {
        &self.aggregate
    }
}
