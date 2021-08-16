use std::{
    collections::HashMap,
    marker::PhantomData,
};

use crate::{
    aggregates::{
        AggregateContext,
        IAggregate,
    },
    errors::AggregateError,
    queries::IQueryProcessor,
    stores::IEventStore,
};

/// This is the base framework for applying commands to produce
/// events.
///
/// In [Domain Driven Design](https://en.wikipedia.org/wiki/Domain-driven_design)
/// we require that changes are made only after loading the entire
/// `Aggregate` in order to ensure that the full context is
/// understood. With event-sourcing this means:
/// 1. loading all previous events for the aggregate instance
/// 2. applying these events, in order, to a new `Aggregate`
/// 3. using the recreated `Aggregate` to handle an inbound `Command`
/// 4. persisting any generated events or rolling back on an error
///
/// To manage these tasks we use a `Repository`.
pub struct Repository<A, ES>
where
    A: IAggregate,
    ES: IEventStore<A>, {
    store: ES,
    query_processors: Vec<Box<dyn IQueryProcessor<A>>>,
    _phantom: PhantomData<AggregateContext<A>>,
}

impl<A, ES> Repository<A, ES>
where
    A: IAggregate,
    ES: IEventStore<A>,
{
    /// Creates new framework for dispatching commands using the
    /// provided elements.
    pub fn new(
        store: ES,
        query_processors: Vec<Box<dyn IQueryProcessor<A>>>,
    ) -> Repository<A, ES> {
        Repository {
            store,
            query_processors,
            _phantom: PhantomData,
        }
    }

    /// This applies a command to an aggregate. Executing a command
    /// in this way is the only way to make any change to
    /// the state of an aggregate.
    ///
    /// An error while processing will result in no events committed
    /// and an AggregateError being returned.
    ///
    /// If successful the events produced will be applied to the
    /// configured `QueryProcessor`s.
    ///
    /// # Error
    /// If an error is generated while processing the command this
    /// will be returned.
    pub fn execute(
        &mut self,
        aggregate_id: &str,
        command: A::Command,
    ) -> Result<(), AggregateError> {
        self.execute_with_metadata(
            aggregate_id,
            command,
            HashMap::new(),
        )
    }

    /// This applies a command to an aggregate along with associated
    /// metadata. Executing a command in this way to make any
    /// change to the state of an aggregate.
    ///
    /// A `Hashmap<String,String>` is supplied with any contextual
    /// information that should be associated with this change.
    /// This metadata will be attached to any produced events and is
    /// meant to assist in debugging and auditing. Common information
    /// might include:
    /// - time of commit
    /// - user making the change
    /// - application version
    ///
    /// An error while processing will result in no events committed
    /// and an AggregateError being returned.
    ///
    /// If successful the events produced will be applied to the
    /// configured `QueryProcessor`s.
    pub fn execute_with_metadata(
        &mut self,
        aggregate_id: &str,
        command: A::Command,
        metadata: HashMap<String, String>,
    ) -> Result<(), AggregateError> {
        let aggregate_context =
            match self.store.load_aggregate(&aggregate_id) {
                Ok(x) => x,
                Err(e) => {
                    return Err(e);
                },
            };

        let events = match aggregate_context
            .aggregate
            .handle(command)
        {
            Ok(x) => x,
            Err(e) => {
                return Err(e);
            },
        };

        let event_contexts = match self.store.commit(
            events,
            aggregate_context,
            metadata,
        ) {
            Ok(x) => x,
            Err(e) => {
                return Err(e);
            },
        };

        let dispatch_events = event_contexts.as_slice();

        for processor in &mut self.query_processors {
            match processor.dispatch(&aggregate_id, &dispatch_events)
            {
                Ok(_) => {},
                Err(e) => {
                    return Err(AggregateError::new(
                        e.to_string().as_str(),
                    ))
                },
            }
        }

        Ok(())
    }
}