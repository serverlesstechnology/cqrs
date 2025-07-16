use crate::aggregate::Aggregate;
use crate::query::Query;
use crate::store::AggregateContext;
use crate::store::EventStore;
use crate::test::AggregateResultValidator;
use crate::EventEnvelope;

/// Holds the initial event state of an aggregate and accepts a command.
pub struct AggregateTestExecutor<A, AC, S>
where
    A: Aggregate,
    AC: AggregateContext<A>,
    S: EventStore<A, AC = AC>,
{
    events: Vec<A::Event>,
    service: A::Services,
    queries: Vec<Box<dyn Query<A>>>,
    context_store: Option<(AC, S)>,
}

impl<A, AC, S> AggregateTestExecutor<A, AC, S>
where
    A: Aggregate,
    AC: AggregateContext<A>,
    S: EventStore<A, AC = AC>,
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
        let result = when::<A, AC, S>(
            self.events,
            command,
            self.service,
            self.queries,
            self.context_store,
        );
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
        let events = self.events;
        let service = self.service;
        let queries = self.queries;
        let context_store = self.context_store;
        for event in events {
            aggregate.apply(event);
        }
        let result = aggregate.handle(command, &service).await;
        if let Ok(resultant_events) = &result {
            let committed_events = commit_events(resultant_events, context_store).await;
            dispatch_events(&queries, &committed_events).await;
        }
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
        let queries = self.queries;
        let context_store = self.context_store;
        Self {
            events,
            service,
            queries,
            context_store,
        }
    }

    pub(crate) fn new(
        events: Vec<A::Event>,
        service: A::Services,
        queries: Vec<Box<dyn Query<A>>>,
        context_store: Option<(AC, S)>,
    ) -> Self {
        Self {
            events,
            service,
            queries,
            context_store,
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn when<A, AC, S>(
    events: Vec<A::Event>,
    command: A::Command,
    service: A::Services,
    queries: Vec<Box<dyn Query<A>>>,
    context_store: Option<(AC, S)>,
) -> Result<Vec<A::Event>, A::Error>
where
    A: Aggregate,
    AC: AggregateContext<A>,
    S: EventStore<A, AC = AC>,
{
    let mut aggregate = A::default();
    for event in events {
        aggregate.apply(event);
    }
    let resultant_events = aggregate.handle(command, &service).await?;
    let committed_events = commit_events(&resultant_events, context_store).await;
    dispatch_events(&queries, &committed_events).await;
    Ok(resultant_events)
}

async fn commit_events<A, AC, S>(
    events: &[A::Event],
    context_store: Option<(AC, S)>,
) -> Vec<EventEnvelope<A>>
where
    A: Aggregate,
    AC: AggregateContext<A>,
    S: EventStore<A, AC = AC>,
{
    let mut wrapped_events = Vec::default();

    if let Some((ctx, store)) = context_store {
        let events = Vec::from(events);
        let mut events = store
            .commit(events, ctx, std::collections::HashMap::default())
            .await
            .expect("persist events in AggregateTestExecutor should be successful");
        wrapped_events.append(&mut events);
    }

    wrapped_events
}

async fn dispatch_events<A: Aggregate>(
    queries: &Vec<Box<dyn Query<A>>>,
    events: &Vec<EventEnvelope<A>>,
) {
    const AGGREGATE_ID: &str = "test_executor_aggregate";

    for query in queries {
        let dispatch_events = events.as_slice();
        query.dispatch(AGGREGATE_ID, dispatch_events).await;
    }
}
