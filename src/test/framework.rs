use crate::aggregate::Aggregate;
use crate::mem_store::MemStore;
use crate::mem_store::MemStoreAggregateContext;
use crate::query::Query;
use crate::store::{AggregateContext, EventStore};
use crate::test::AggregateTestExecutor;

/// A framework for rigorously testing the aggregate logic, one of the *most important*
/// parts of any DDD system.
pub type TestFramework<A> = GenericTestFramework<A, MemStoreAggregateContext<A>, MemStore<A>>;

/// The framework implementation.
pub struct GenericTestFramework<A, AC, S>
where
    A: Aggregate,
    AC: AggregateContext<A>,
    S: EventStore<A, AC = AC>,
{
    service: A::Services,
    queries: Vec<Box<dyn Query<A>>>,
    context_store: Option<(AC, S)>,
}

impl<A: Aggregate> GenericTestFramework<A, MemStoreAggregateContext<A>, MemStore<A>> {
    /// Create a test framework using the provided service. No queries or event store are defined.
    pub fn with(service: A::Services) -> Self {
        let queries = Vec::default();
        let context_store = None;
        Self {
            service,
            queries,
            context_store,
        }
    }

    /// Use an event store within the current test framework.
    pub fn using_context_and_store<SO, ACO>(
        self,
        context: ACO,
        store: SO,
    ) -> GenericTestFramework<A, ACO, SO>
    where
        ACO: AggregateContext<A>,
        SO: EventStore<A, AC = ACO>,
    {
        let service = self.service;
        let queries = self.queries;
        let context_store = Some((context, store));
        GenericTestFramework {
            service,
            queries,
            context_store,
        }
    }

    /// Use a [MemStore] event store within the current test framework.
    pub fn using_mem_store(
        self,
    ) -> GenericTestFramework<A, MemStoreAggregateContext<A>, MemStore<A>> {
        self.using_context_and_store(MemStoreAggregateContext::default(), MemStore::default())
    }
}

impl<A, AC, S> GenericTestFramework<A, AC, S>
where
    A: Aggregate,
    AC: AggregateContext<A>,
    S: EventStore<A, AC = AC>,
{
    /// Add a query into the current test framework. An event store must be defined
    /// before providing pre-conditions with ([given_no_previous_events] / [given]).
    pub fn and_query(self, query: Box<dyn Query<A>>) -> Self {
        let service = self.service;
        let mut queries = self.queries;
        queries.push(query);
        let context_store = self.context_store;
        Self {
            service,
            queries,
            context_store,
        }
    }

    /// Add all queries into the current test framework. An event store must be defined
    /// before providing pre-conditions ([given_no_previous_events] / [given]).
    pub fn and_queries(self, queries: Vec<Box<dyn Query<A>>>) -> Self {
        queries.into_iter().fold(self, |acc, q| acc.and_query(q))
    }

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
    pub fn given_no_previous_events(self) -> AggregateTestExecutor<A, AC, S> {
        AggregateTestExecutor::new(Vec::new(), self.service, self.queries, self.context_store)
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
    pub fn given(self, events: Vec<A::Event>) -> AggregateTestExecutor<A, AC, S> {
        AggregateTestExecutor::new(events, self.service, self.queries, self.context_store)
    }
}
