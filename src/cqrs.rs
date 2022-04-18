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
/// 1. Applying any generated events to the `Aggregate` (see `apply` method in this trait).
/// 1. Persisting any generated events or rolling back in the event of an error.
///
/// To manage these tasks we use a `CqrsFramework`.
///
pub struct CqrsFramework<A, ES>
where
    A: Aggregate,
    ES: EventStore<A>,
{
    store: ES,
    query_processors: Vec<Box<dyn Query<A>>>,
}

impl<A, ES> CqrsFramework<A, ES>
where
    A: Aggregate,
    ES: EventStore<A>,
{
    /// Creates new framework for dispatching commands using the provided elements.
    /// Takes an `EventStore` and a vector of queries.
    ///
    /// For a simple in-memory `EventStore` suitable for experimentation or testing see
    /// [MemStore](mem_store/struct.MemStore.html).
    ///
    /// ```
    /// # use cqrs_es::doc::MyAggregate;
    /// use cqrs_es::CqrsFramework;
    /// use cqrs_es::mem_store::MemStore;
    ///
    /// let store = MemStore::<MyAggregate>::default();
    /// let cqrs = CqrsFramework::new(store, vec![]);
    /// ```
    /// For production uses a persistent event store
    /// using a backing database is needed, such as in the available persistence crates:
    /// - [PostgreSQL](https://www.postgresql.org/) - [postgres-es](https://crates.io/crates/postgres-es)
    /// - [MySQL](https://www.mysql.com/) - [mysql-es](https://crates.io/crates/mysql-es)
    /// - [DynamoDb](https://aws.amazon.com/dynamodb/) - [dynamo-es](https://crates.io/crates/dynamo-es)
    ///
    pub fn new(store: ES, query_processors: Vec<Box<dyn Query<A>>>) -> CqrsFramework<A, ES>
    where
        A: Aggregate,
        ES: EventStore<A>,
    {
        CqrsFramework {
            store,
            query_processors,
        }
    }
    /// This applies a command to an aggregate. Executing a command
    /// in this way is the only way to make any change to
    /// the state of an aggregate.
    ///
    /// An error while processing will result in no events committed and
    /// an AggregateError being returned.
    ///
    /// If successful the events produced will be persisted in the backing `EventStore`
    /// before being applied to any configured `QueryProcessor`s.
    ///
    /// ```
    /// # use cqrs_es::CqrsFramework;
    /// # use cqrs_es::doc::{MyAggregate,MyCommands};
    /// # use cqrs_es::mem_store::MemStore;
    /// # use std::collections::HashMap;
    /// # use chrono;
    /// # async fn test(cqrs: CqrsFramework<MyAggregate,MemStore<MyAggregate>>) {
    /// let command = MyCommands::DoSomething;
    ///
    /// cqrs.execute("agg-id-F39A0C", command).await;
    /// # }
    /// ```
    pub async fn execute(
        &self,
        aggregate_id: &str,
        command: A::Command,
    ) -> Result<(), AggregateError<A::Error>> {
        self.execute_with_metadata(aggregate_id, command, HashMap::new())
            .await
    }

    /// This applies a command to an aggregate along with associated metadata. Executing a command
    /// in this way to make any change to the state of an aggregate.
    ///
    /// A `Hashmap<String,String>` is supplied with any contextual information that should be
    /// associated with this change. This metadata will be attached to any produced events and is
    /// meant to assist in debugging and auditing. Common information might include:
    /// - time of commit
    /// - user making the change
    /// - application version
    ///
    /// An error while processing will result in no events committed and
    /// an AggregateError being returned.
    ///
    /// If successful the events produced will be persisted in the backing `EventStore`
    /// before being applied to any configured `QueryProcessor`s.
    ///
    /// ```
    /// # use cqrs_es::CqrsFramework;
    /// # use cqrs_es::doc::{MyAggregate,MyCommands};
    /// # use cqrs_es::mem_store::MemStore;
    /// # use std::collections::HashMap;
    /// # use chrono;
    /// # async fn test(cqrs: CqrsFramework<MyAggregate,MemStore<MyAggregate>>) {
    /// let command = MyCommands::DoSomething;
    /// let mut metadata = HashMap::new();
    /// metadata.insert("time".to_string(), chrono::Utc::now().to_rfc3339());
    ///
    /// cqrs.execute_with_metadata("agg-id-F39A0C", command, metadata).await;
    /// # }
    /// ```
    pub async fn execute_with_metadata(
        &self,
        aggregate_id: &str,
        command: A::Command,
        metadata: HashMap<String, String>,
    ) -> Result<(), AggregateError<A::Error>> {
        let aggregate_context = self.store.load_aggregate(aggregate_id).await?;
        let aggregate = aggregate_context.aggregate();
        let resultant_events = match aggregate.handle(command).await {
            Ok(events) => events,
            Err(err) => return Err(AggregateError::UserError(err)),
        };
        let committed_events = self
            .store
            .commit(resultant_events, aggregate_context, metadata)
            .await?;
        for processor in &self.query_processors {
            let dispatch_events = committed_events.as_slice();
            processor.dispatch(aggregate_id, dispatch_events).await;
        }
        Ok(())
    }
}
