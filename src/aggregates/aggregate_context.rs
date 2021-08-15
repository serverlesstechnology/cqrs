use std::fmt::Debug;

use super::i_aggregate::IAggregate;

/// Returns the aggregate and context around it that is needed when
/// committing events in an event store implementation.
#[derive(Debug, Clone)]
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