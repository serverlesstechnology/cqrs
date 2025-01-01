use crate::aggregate::Aggregate;
use crate::test::AggregateResultValidator;

/// Holds the initial event state of an aggregate and accepts a command.
pub struct AggregateTestExecutor<A>
where
    A: Aggregate,
{
    events: Vec<A::Event>,
    service: A::Services,
}

impl<A> AggregateTestExecutor<A>
where
    A: Aggregate,
{
    /// Consumes a command and provides a validator object to test against.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyCommands, MyService};
    /// use cqrs_es::test::TestFramework;
    ///
    /// let executor = TestFramework::<MyAggregate>::with(MyService)
    ///     .given_no_previous_events();
    ///
    /// let validator = executor.when(MyCommands::DoSomething);
    /// ```
    ///
    /// For `async` tests use `when_async` instead.
    pub fn when(self, command: A::Command) -> AggregateResultValidator<A> {
        let result = when::<A>(self.events, command, self.service);
        AggregateResultValidator::new(result)
    }

    /// Consumes a command in an `async` test and provides a validator object
    /// to test against.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyCommands, MyService};
    /// use cqrs_es::test::TestFramework;
    ///
    /// #[tokio::test]
    /// async fn test() {
    ///     let executor = TestFramework::<MyAggregate>::with(MyService)
    ///         .given_no_previous_events();
    ///
    ///     let validator = executor.when_async(MyCommands::DoSomething).await;
    /// }
    /// ```
    pub async fn when_async(self, command: A::Command) -> AggregateResultValidator<A> {
        let mut aggregate = A::default();
        for event in self.events {
            aggregate.apply(event);
        }
        let result = aggregate.handle(command, &self.service).await;
        AggregateResultValidator::new(result)
    }

    /// Adds additional events to an aggregate test.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyEvents, MyService};
    /// use cqrs_es::test::TestFramework;
    ///
    /// let executor = TestFramework::<MyAggregate>::with(MyService)
    ///     .given(vec![MyEvents::SomethingWasDone])
    ///     .and(vec![MyEvents::SomethingElseWasDone]);
    /// ```
    #[must_use]
    pub fn and(self, new_events: Vec<A::Event>) -> Self {
        let mut events = self.events;
        events.extend(new_events);
        let service = self.service;
        Self { events, service }
    }

    pub(crate) fn new(events: Vec<A::Event>, service: A::Services) -> Self {
        Self { events, service }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn when<A: Aggregate>(
    events: Vec<A::Event>,
    command: A::Command,
    service: A::Services,
) -> Result<Vec<A::Event>, A::Error> {
    let mut aggregate = A::default();
    for event in events {
        aggregate.apply(event);
    }
    aggregate.handle(command, &service).await
}
