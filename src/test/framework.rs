use crate::aggregate::Aggregate;
use crate::test::AggregateTestExecutor;

/// A framework for rigorously testing the aggregate logic, one of the *most important*
/// parts of any DDD system.
pub struct TestFramework<A: Aggregate> {
    service: A::Services,
}

impl<A: Aggregate> TestFramework<A> {
    /// Create a test framework using the provided service.
    pub fn with(service: A::Services) -> Self {
        Self { service }
    }
}

impl<A> TestFramework<A>
where
    A: Aggregate,
{
    /// Initiates an aggregate test with no previous events.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyService};
    /// use cqrs_es::test::TestFramework;
    ///
    /// let executor = TestFramework::<MyAggregate>::with(MyService)
    ///     .given_no_previous_events();
    /// ```
    #[must_use]
    pub fn given_no_previous_events(self) -> AggregateTestExecutor<A> {
        AggregateTestExecutor::new(Vec::new(), self.service)
    }
    /// Initiates an aggregate test with a collection of previous events.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyEvents, MyService};
    /// use cqrs_es::test::TestFramework;
    ///
    /// let executor = TestFramework::<MyAggregate>::with(MyService)
    ///     .given(vec![MyEvents::SomethingWasDone]);
    /// ```
    #[must_use]
    pub fn given(self, events: Vec<A::Event>) -> AggregateTestExecutor<A> {
        AggregateTestExecutor::new(events, self.service)
    }
}
