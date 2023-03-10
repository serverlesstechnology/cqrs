use std::collections::HashMap;

use crate::query::Query;
use crate::store::EventStore;
use crate::Aggregate;
use crate::{AggregateContext, AggregateError};

/// This is the base framework for applying commands to produce events.
///
/// In [Domain Driven Design](https://en.wikipedia.org/wiki/Domain-driven_design) we require that
/// changes are made only after loading the entire `Aggregate` in order to ensure that the full
/// context is understood.
/// With event-sourcing this means:
/// 1. Loading all previous events for the aggregate instance.
/// 1. Applying these events, in order, to a new `Aggregate` in order to reach the correct state.
/// 1. Using the recreated `Aggregate` to handle an inbound `Command` producing events or an error
/// (see `handle` method in this trait).
/// 1. Persisting any generated events or roll back in the event of an error.
///
/// To manage these tasks we use a `CqrsFramework`.
///
pub struct CqrsFramework<A, ES>
where
    A: Aggregate,
    ES: EventStore<A>,
{
    store: ES,
    queries: Vec<Box<dyn Query<A>>>,
    service: A::Services,
}

impl<A, ES> CqrsFramework<A, ES>
where
    A: Aggregate,
    ES: EventStore<A>,
{
    /// Creates new framework for dispatching commands using the provided elements.
    /// Takes an implementation of an `EventStore`, a vector of queries and a set of services
    /// to be used within the command handler.
    ///
    /// For a simple in-memory `EventStore` suitable for experimentation or testing see
    /// [MemStore](mem_store/struct.MemStore.html).
    ///
    /// ```rust
    /// # use cqrs_es::doc::{MyAggregate, MyService};
    /// use cqrs_es::CqrsFramework;
    /// use cqrs_es::mem_store::MemStore;
    ///
    /// let store = MemStore::<MyAggregate>::default();
    /// let queries = vec![];
    /// let service = MyService::default();
    ///
    /// let cqrs = CqrsFramework::new(store, queries, service);
    /// ```
    /// For production uses a
    /// [persistent event store](https://docs.rs/cqrs-es/latest/cqrs_es/persist/struct.PersistedEventStore.html)
    /// using a backing database is needed, such as in the available persistence crates:
    /// - [PostgreSQL](https://www.postgresql.org/) - [postgres-es](https://crates.io/crates/postgres-es)
    /// - [MySQL](https://www.mysql.com/) - [mysql-es](https://crates.io/crates/mysql-es)
    /// - [DynamoDb](https://aws.amazon.com/dynamodb/) - [dynamo-es](https://crates.io/crates/dynamo-es)
    ///
    pub fn new(store: ES, queries: Vec<Box<dyn Query<A>>>, service: A::Services) -> Self
    where
        A: Aggregate,
        ES: EventStore<A>,
    {
        Self {
            store,
            queries,
            service,
        }
    }
    /// Appends an additional query to the framework.
    /// ```rust
    /// # use cqrs_es::doc::{MyAggregate, MyQuery, MyService};
    /// use cqrs_es::CqrsFramework;
    /// use cqrs_es::mem_store::MemStore;
    ///
    /// let store = MemStore::<MyAggregate>::default();
    /// let queries = vec![];
    /// let service = MyService::default();
    ///
    /// let cqrs = CqrsFramework::new(store, queries, service)
    ///     .append_query(Box::new(MyQuery::default()));
    /// ```
    pub fn append_query(self, query: Box<dyn Query<A>>) -> Self
    where
        A: Aggregate,
        ES: EventStore<A>,
    {
        let mut queries = self.queries;
        queries.push(query);
        Self {
            store: self.store,
            queries,
            service: self.service,
        }
    }
    /// This applies a command to an aggregate. Executing a command
    /// in this way is the only way to make changes to
    /// the state of an aggregate in CQRS.
    ///
    /// An error while processing will result in no events committed and
    /// an [`AggregateError`](https://docs.rs/cqrs-es/latest/cqrs_es/enum.AggregateError.html)
    /// being returned.
    ///
    /// If successful the events produced will be persisted in the backing `EventStore`
    /// before being applied to any configured `QueryProcessor`s.
    ///
    /// ```
    /// # use cqrs_es::{AggregateError, CqrsFramework};
    /// # use cqrs_es::doc::{MyAggregate, MyCommands, MyUserError};
    /// # use cqrs_es::mem_store::MemStore;
    /// # use std::collections::HashMap;
    /// # use chrono;
    /// type MyFramework = CqrsFramework<MyAggregate,MemStore<MyAggregate>>;
    ///
    /// async fn do_something(cqrs: MyFramework) -> Result<(),AggregateError<MyUserError>> {
    ///     let command = MyCommands::DoSomething;
    ///
    ///     cqrs.execute("agg-id-F39A0C", command).await
    /// }
    /// ```
    pub async fn execute(
        &self,
        aggregate_id: &str,
        command: A::Command,
    ) -> Result<(), AggregateError<A::Error>> {
        self.execute_with_metadata(aggregate_id, command, HashMap::new())
            .await
    }

    /// This applies a command to an aggregate. Executing a command
    /// in this way is the only way to make changes to
    /// the state of an aggregate in CQRS.
    ///
    /// A `Hashmap<String,String>` is supplied with any contextual information that should be
    /// associated with this change. This metadata will be attached to any produced events and is
    /// meant to assist in debugging and auditing. Common information might include:
    /// - time of commit
    /// - user making the change
    /// - application version
    ///
    /// An error while processing will result in no events committed and
    /// an [`AggregateError`](https://docs.rs/cqrs-es/latest/cqrs_es/enum.AggregateError.html)
    /// being returned.
    ///
    /// If successful the events produced will be persisted in the backing `EventStore`
    /// before being applied to any configured `QueryProcessor`s.
    ///
    /// ```
    /// # use cqrs_es::{AggregateError, CqrsFramework};
    /// # use cqrs_es::doc::{MyAggregate, MyCommands, MyUserError};
    /// # use cqrs_es::mem_store::MemStore;
    /// # use std::collections::HashMap;
    /// # use chrono;
    /// type MyFramework = CqrsFramework<MyAggregate,MemStore<MyAggregate>>;
    ///
    /// async fn do_something(cqrs: MyFramework) -> Result<(),AggregateError<MyUserError>>  {
    ///     let command = MyCommands::DoSomething;
    ///     let metadata = HashMap::from([("time".to_string(), chrono::Utc::now().to_rfc3339())]);
    ///
    ///     cqrs.execute_with_metadata("agg-id-F39A0C", command, metadata).await
    /// }
    /// ```
    pub async fn execute_with_metadata(
        &self,
        aggregate_id: &str,
        command: A::Command,
        metadata: HashMap<String, String>,
    ) -> Result<(), AggregateError<A::Error>> {
        let aggregate_context = self.store.load_aggregate(aggregate_id).await?;
        let aggregate = aggregate_context.aggregate();
        let resultant_events = aggregate
            .handle(command, &self.service)
            .await
            .map_err(AggregateError::UserError)?;
        let committed_events = self
            .store
            .commit(resultant_events, aggregate_context, metadata)
            .await?;
        for processor in &self.queries {
            let dispatch_events = committed_events.as_slice();
            processor.dispatch(aggregate_id, dispatch_events).await;
        }
        Ok(())
    }
}
