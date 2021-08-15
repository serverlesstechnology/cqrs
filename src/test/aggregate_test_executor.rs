use crate::aggregates::IAggregate;

use super::aggregate_result_validator::AggregateResultValidator;

/// Holds the initial event state of an aggregate and accepts a
/// command.
pub struct AggregateTestExecutor<A: IAggregate> {
    pub events: Vec<A::Event>,
}

impl<A: IAggregate> AggregateTestExecutor<A> {
    /// Consumes a command and using the state details previously
    /// passed provides a validator object to test against.
    pub fn when(
        self,
        command: A::Command,
    ) -> AggregateResultValidator<A> {
        let mut aggregate = A::default();
        for event in self.events {
            aggregate.apply(&event);
        }
        let result = aggregate.handle(command);
        AggregateResultValidator { result }
    }
}
