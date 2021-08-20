use log::{
    debug,
    error,
    trace,
};
use std::{
    collections::HashMap,
    marker::PhantomData,
};

use cqrs_es2_core::{
    Error,
    IAggregate,
    ICommand,
    IEvent,
};

use super::{
    i_event_dispatcher::IEventDispatcher,
    i_event_store::IEventStore,
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
pub struct Repository<
    C: ICommand,
    E: IEvent,
    A: IAggregate<C, E>,
    ES: IEventStore<C, E, A>,
> {
    store: ES,
    dispatchers: Vec<Box<dyn IEventDispatcher<C, E>>>,
    _phantom: PhantomData<A>,
}

impl<
        C: ICommand,
        E: IEvent,
        A: IAggregate<C, E>,
        ES: IEventStore<C, E, A>,
    > Repository<C, E, A, ES>
{
    /// Creates new framework for dispatching commands using the
    /// provided elements.
    pub fn new(
        store: ES,
        dispatchers: Vec<Box<dyn IEventDispatcher<C, E>>>,
    ) -> Self {
        let x = Self {
            store,
            dispatchers,
            _phantom: PhantomData,
        };

        trace!("Created new Repository");

        x
    }

    /// This applies a command to an aggregate. Executing a command
    /// in this way is the only way to make any change to
    /// the state of an aggregate.
    ///
    /// An error while processing will result in no events committed
    /// and an Error being returned.
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
        command: C,
    ) -> Result<(), Error> {
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
    /// and an Error being returned.
    ///
    /// If successful the events produced will be applied to the
    /// configured `QueryProcessor`s.
    pub fn execute_with_metadata(
        &mut self,
        aggregate_id: &str,
        command: C,
        metadata: HashMap<String, String>,
    ) -> Result<(), Error> {
        trace!(
            "Applying command '{:?}' to aggregate '{}' with \
             metadata '{:?}'",
            &command,
            &aggregate_id,
            &metadata
        );

        let aggregate_context =
            match self.store.load_aggregate(&aggregate_id) {
                Ok(x) => x,
                Err(e) => {
                    error!(
                        "Loading aggregate '{}' returned error '{}'",
                        &aggregate_id,
                        e.to_string()
                    );
                    return Err(e);
                },
            };

        let events = match aggregate_context
            .aggregate
            .handle(command.clone())
        {
            Ok(x) => x,
            Err(e) => {
                error!(
                    "Handling command '{:?}' for aggregate '{}' \
                     returned error '{}'",
                    &command,
                    &aggregate_id,
                    e.to_string()
                );
                return Err(e);
            },
        };

        let event_contexts = match self.store.commit(
            events,
            aggregate_context,
            metadata.clone(),
        ) {
            Ok(x) => x,
            Err(e) => {
                error!(
                    "Committing events returned error '{}'",
                    e.to_string()
                );
                return Err(e);
            },
        };

        let dispatch_events = event_contexts.as_slice();

        for processor in &mut self.dispatchers {
            match processor.dispatch(&aggregate_id, &dispatch_events)
            {
                Ok(_) => {},
                Err(e) => {
                    error!(
                        "dispatcher returned error '{}'",
                        e.to_string()
                    );
                    return Err(Error::new(e.to_string().as_str()));
                },
            }
        }

        debug!(
            "Successfully applied command '{:?}' to aggregate '{}' \
             with metadata '{:?}'",
            &command, &aggregate_id, &metadata
        );

        Ok(())
    }
}
