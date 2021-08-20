use log::trace;
use std::{
    fmt::Debug,
    marker::PhantomData,
};

use crate::{
    commands::ICommand,
    events::IEvent,
};

use super::i_aggregate::IAggregate;

/// Returns the aggregate and context around it that is needed when
/// committing events in an event store implementation.
#[derive(Debug, PartialEq, Clone)]
pub struct AggregateContext<
    C: ICommand,
    E: IEvent,
    A: IAggregate<C, E>,
> {
    /// The aggregate ID of the aggregate instance that has been
    /// loaded.
    pub aggregate_id: String,

    /// The current state of the aggregate instance.
    pub aggregate: A,

    /// The last committed event sequence number for this aggregate
    /// instance.
    pub current_sequence: usize,

    _phantom: PhantomData<(C, E)>,
}

impl<C: ICommand, E: IEvent, A: IAggregate<C, E>>
    AggregateContext<C, E, A>
{
    /// constructor
    pub fn new(
        aggregate_id: String,
        aggregate: A,
        current_sequence: usize,
    ) -> Self {
        let x = Self {
            aggregate_id,
            aggregate,
            current_sequence,
            _phantom: PhantomData,
        };

        trace!("Created new {:?}", x,);

        x
    }
}
