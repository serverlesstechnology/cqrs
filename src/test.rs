use std::marker::PhantomData;

use crate::aggregate::Aggregate;

/// A framework for rigorously testing the aggregate logic, one of the ***most important***
/// parts of any DDD system.
///
/// ```
/// # use cqrs_es::doc::MyAggregate;
/// use cqrs_es::test::TestFramework;
///
/// let framework = TestFramework::<MyAggregate>::default();
/// ```
pub struct TestFramework<A> {
    _phantom: PhantomData<A>,
}

impl<A> TestFramework<A>
where
    A: Aggregate,
{
    /// Initiates an aggregate test with no previous events.
    ///
    /// ```
    /// # use cqrs_es::doc::MyAggregate;
    /// use cqrs_es::test::TestFramework;
    ///
    /// let executor = TestFramework::<MyAggregate>::default()
    ///     .given_no_previous_events();
    /// ```
    #[must_use]
    pub fn given_no_previous_events(&self) -> AggregateTestExecutor<A> {
        AggregateTestExecutor { events: Vec::new() }
    }
    /// Initiates an aggregate test with a collection of previous events.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyEvents};
    /// use cqrs_es::test::TestFramework;
    ///
    /// let executor = TestFramework::<MyAggregate>::default()
    ///     .given(vec![MyEvents::SomethingWasDone]);
    /// ```
    #[must_use]
    pub fn given(&self, events: Vec<A::Event>) -> AggregateTestExecutor<A> {
        AggregateTestExecutor { events }
    }
}

impl<A> Default for TestFramework<A>
where
    A: Aggregate,
{
    fn default() -> Self {
        TestFramework {
            _phantom: PhantomData,
        }
    }
}

/// Holds the initial event state of an aggregate and accepts a command.
pub struct AggregateTestExecutor<A>
where
    A: Aggregate,
{
    events: Vec<A::Event>,
}

impl<A> AggregateTestExecutor<A>
where
    A: Aggregate,
{
    /// Consumes a command and using the state details previously passed provides a validator object
    /// to test against.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyCommands};
    /// use cqrs_es::test::TestFramework;
    ///
    /// let executor = TestFramework::<MyAggregate>::default().given_no_previous_events();
    ///
    /// let validator = executor.when(MyCommands::DoSomething);
    /// ```
    pub fn when(self, command: A::Command) -> AggregateResultValidator<A> {
        let result = when::<A>(self.events, command);
        AggregateResultValidator { result }
    }
}

#[tokio::main]
async fn when<A: Aggregate>(
    events: Vec<A::Event>,
    command: A::Command,
) -> Result<Vec<A::Event>, A::Error> {
    let mut aggregate = A::default();
    for event in events {
        aggregate.apply(event);
    }
    aggregate.handle(command).await
}

/// Validation object for the `TestFramework` package.
pub struct AggregateResultValidator<A>
where
    A: Aggregate,
{
    result: Result<Vec<A::Event>, A::Error>,
}

impl<A: Aggregate> AggregateResultValidator<A> {
    /// Verifies that the expected events have been produced by the command.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyCommands, MyEvents};
    /// # async fn test() {
    /// use cqrs_es::test::TestFramework;
    ///
    /// let validator = TestFramework::<MyAggregate>::default()
    ///     .given_no_previous_events()
    ///     .when(MyCommands::DoSomething);
    ///
    /// validator.then_expect_events(vec![MyEvents::SomethingWasDone]);
    /// # }
    /// ```
    pub fn then_expect_events(self, expected_events: Vec<A::Event>) {
        let events = match self.result {
            Ok(expected_events) => expected_events,
            Err(err) => {
                panic!("expected success, received aggregate error: '{}'", err);
            }
        };
        assert_eq!(&events[..], &expected_events[..]);
    }

    /// Verifies that the result is a `UserError` and returns the internal error payload for
    /// further validation.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyCommands, MyEvents, MyUserError};
    /// use cqrs_es::test::TestFramework;
    ///
    /// let validator = TestFramework::<MyAggregate>::default()
    ///     .given_no_previous_events()
    ///     .when(MyCommands::BadCommand);
    ///
    /// let expected = MyUserError("the expected error message".to_string());
    /// assert_eq!(expected,validator.then_expect_error());
    /// ```
    pub fn then_expect_error(self) -> A::Error {
        match self.result {
            Ok(events) => {
                panic!("expected error, received events: '{:?}'", events);
            }
            Err(err) => err,
        }
    }

    /// Verifies that an `AggregateError` with the expected message is produced with the command.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyCommands, MyEvents};
    /// use cqrs_es::test::TestFramework;
    ///
    /// let validator = TestFramework::<MyAggregate>::default()
    ///     .given_no_previous_events()
    ///     .when(MyCommands::BadCommand);
    ///
    /// validator.then_expect_error_message("the expected error message");
    /// ```
    pub fn then_expect_error_message(self, error_message: &str) {
        match self.result {
            Ok(events) => {
                panic!("expected error, received events: '{:?}'", events);
            }
            Err(err) => assert_eq!(err.to_string(), error_message.to_string()),
        };
    }
}
