use std::{
    collections::HashMap,
    marker::PhantomData,
};

use crate::{
    aggregates::{
        Aggregate,
        AggregateError,
    },
    queries::QueryProcessor,
    stores::{
        AggregateContext,
        EventStore,
    },
};

/// This is the base framework for applying commands to produce
/// events.
///
/// In [Domain Driven Design](https://en.wikipedia.org/wiki/Domain-driven_design) we require that
/// changes are made only after loading the entire `Aggregate` in
/// order to ensure that the full context is understood.
/// With event-sourcing this means:
/// 1. loading all previous events for the aggregate instance
/// 1. applying these events, in order, to a new `Aggregate`
/// 1. using the recreated `Aggregate` to handle an inbound `Command`
/// 1. persisting any generated events or rolling back on an error
///
/// To manage these tasks we use a `CqrsFramework`.
pub struct CqrsFramework<A, ES, AC>
where
    A: Aggregate,
    ES: EventStore<A, AC>,
    AC: AggregateContext<A>, {
    store: ES,
    query_processors: Vec<Box<dyn QueryProcessor<A>>>,
    _phantom: PhantomData<AC>,
}

impl<A, ES, AC> CqrsFramework<A, ES, AC>
where
    A: Aggregate,
    ES: EventStore<A, AC>,
    AC: AggregateContext<A>,
{
    /// Creates new framework for dispatching commands using the
    /// provided elements.
    pub fn new(
        store: ES,
        query_processors: Vec<Box<dyn QueryProcessor<A>>>,
    ) -> CqrsFramework<A, ES, AC>
    where
        A: Aggregate,
        ES: EventStore<A, AC>,
        AC: AggregateContext<A>, {
        CqrsFramework {
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
            self.store.load_aggregate(aggregate_id);
        let aggregate = aggregate_context.aggregate();
        let resultant_events = aggregate.handle(command)?;
        let committed_events = self.store.commit(
            resultant_events,
            aggregate_context,
            metadata,
        )?;
        for processor in &mut self.query_processors {
            let dispatch_events = committed_events.as_slice();
            processor.dispatch(&aggregate_id, dispatch_events);
        }
        Ok(())
    }
}
