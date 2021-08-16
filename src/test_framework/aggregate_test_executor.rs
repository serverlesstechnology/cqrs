use std::marker::PhantomData;

use crate::{
    aggregates::IAggregate,
    commands::ICommand,
    events::IEvent,
};

use super::aggregate_result_validator::AggregateResultValidator;

/// Holds the initial event state of an aggregate and accepts a
/// command.
pub struct AggregateTestExecutor<
    C: ICommand,
    E: IEvent,
    A: IAggregate<C, E>,
> {
    pub events: Vec<E>,
    _phantom: PhantomData<(C, A)>,
}

impl<C: ICommand, E: IEvent, A: IAggregate<C, E>>
    AggregateTestExecutor<C, E, A>
{
    pub fn new(events: Vec<E>) -> Self {
        Self {
            events,
            _phantom: PhantomData,
        }
    }

    /// Consumes a command and using the state details previously
    /// passed provides a validator object to test against.
    pub fn when(
        self,
        command: C,
    ) -> AggregateResultValidator<E> {
        let mut aggregate = A::default();
        for event in self.events {
            aggregate.apply(&event);
        }
        let result = aggregate.handle(command);
        AggregateResultValidator::new(result)
    }
}
