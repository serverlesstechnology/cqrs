use std::marker::PhantomData;

use crate::aggregate::{Aggregate, AggregateError};
use crate::command::Command;
use crate::event::DomainEvent;

/// A framework for rigorously testing the aggregate logic, one of the **most important**
/// parts of any CQRS system.
pub struct TestFramework<A, E> {
    _phantom: PhantomData<(A, E)>
}

impl<A, E> TestFramework<A, E>
    where A: Aggregate,
          E: DomainEvent<A> {
    /// Initiates an aggregate test with no previous events.
    #[must_use]
    pub fn given_no_previous_events(&self) -> AggregateTestExecutor<A, E> {
        AggregateTestExecutor { events: Vec::new(), _phantom: PhantomData }
    }
    /// Initiates an aggregate test with a collection of previous events.
    #[must_use]
    pub fn given(&self, events: Vec<E>) -> AggregateTestExecutor<A, E> {
        AggregateTestExecutor { events, _phantom: PhantomData }
    }
}

impl<A, E> Default for TestFramework<A, E>
    where A: Aggregate,
          E: DomainEvent<A>
{
    fn default() -> Self {
        TestFramework { _phantom: PhantomData }
    }
}

/// Holds the initial event state of an aggregate and accepts a command.
pub struct AggregateTestExecutor<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    events: Vec<E>,
    _phantom: PhantomData<A>,
}

impl<A, E> AggregateTestExecutor<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A> {
    /// Consumes a command and using the state details previously passed provides a validator object
    /// to test against.
    pub fn when<C: Command<A, E>>(self, command: C) -> AggregateResultValidator<A, E> {
        let mut aggregate = A::default();
        for event in self.events {
            event.apply(&mut aggregate)
        }
        let result = command.handle(&mut aggregate);
        AggregateResultValidator { result, _phantom: PhantomData }
    }
}

/// Validation object for the `TestFramework` package.
pub struct AggregateResultValidator<A, E>
    where A: Aggregate,
          E: DomainEvent<A> {
    result: Result<Vec<E>, AggregateError>,
    _phantom: PhantomData<A>,
}

impl<A, E> AggregateResultValidator<A, E>
    where A: Aggregate,
          E: DomainEvent<A> {
    /// Verifies that the expected events have been produced by the command.
    pub fn then_expect_events(self, expected_events: Vec<E>) {
        let events = match self.result {
            Ok(expected_events) => { expected_events }
            Err(err) => { panic!("expected success, received aggregate error: '{}'", err); }
        };
        assert_eq!(&events[..], &expected_events[..]);
    }
    /// Verifies that an `AggregateError` with the expected message is produced with the command.
    pub fn then_expect_error(self, error_message: &str) {
        match self.result {
            Ok(events) => { panic!("expected error, received events: '{:?}'", events); }
            Err(err) => {
                match err {
                    AggregateError::TechnicalError(err) => {
                        panic!("expected user error but found technical error: {}", err)
                    },
                    AggregateError::UserError(err) => {
                        assert_eq!(err, error_message);
                    },
                }
            }
        };
    }
}
