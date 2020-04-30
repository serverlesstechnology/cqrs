use std::collections::HashMap;
use std::marker::PhantomData;

use crate::aggregate::{Aggregate, AggregateError};
use crate::command::Command;
use crate::event::{DomainEvent, MessageEnvelope};
use crate::query::QueryProcessor;
use crate::store::EventStore;

/// This is the base framework for applying commands to produce events.
///
/// In [Domain Driven Design](https://en.wikipedia.org/wiki/Domain-driven_design) we require that
/// changes are made only after loading the entire `Aggregate` in order to ensure that the full
/// context is understood.
/// With event-sourcing this means:
/// 1. loading all previous events for the aggregate instance
/// 1. applying these events, in order, to a new `Aggregate`
/// 1. using the recreated `Aggregate` to handle an inbound `Command`
/// 1. persisting any generated events or rolling back on an error
///
/// To manage these tasks we use a `CqrsFramework`.
///
pub struct CqrsFramework<A, E, ES>
    where
        A: Aggregate,
        E: DomainEvent<A>,
        ES: EventStore<A, E>,
{
    store: ES,
    query_processors: Vec<Box<dyn QueryProcessor<A, E>>>,
}

impl<A, E, ES> CqrsFramework<A, E, ES>
    where
        A: Aggregate,
        E: DomainEvent<A>,
        ES: EventStore<A, E>
{
    /// Creates new framework for dispatching commands using the provided elements.
    pub fn new(store: ES, query_processors: Vec<Box<dyn QueryProcessor<A, E>>>) -> CqrsFramework<A, E, ES>
        where
            A: Aggregate,
            E: DomainEvent<A>,
            ES: EventStore<A, E>
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
    /// If successful the events produced will be applied to the configured `QueryProcessor`s.
    ///
    /// # Error
    /// If an error is generated while processing the command this will be returned.
    pub fn execute<C: Command<A, E>>(&self, aggregate_id: &str, command: C) -> Result<(), AggregateError> {
        self.execute_with_metadata(aggregate_id, command, HashMap::new())
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
    /// If successful the events produced will be applied to the configured `QueryProcessor`s.
    pub fn execute_with_metadata<C: Command<A, E>>(&self, aggregate_id: &str, command: C, metadata: HashMap<String, String>) -> Result<(), AggregateError> {
        let (aggregate, current_sequence) = self.load_aggregate(aggregate_id);
        let resultant_events = command.handle(&aggregate)?;
        let wrapped_events = self.wrap_events(aggregate_id, current_sequence, resultant_events, metadata);

        let committed_events = <CqrsFramework<A, E, ES>>::duplicate(&wrapped_events);
        self.store.commit(wrapped_events)?;
        for processor in &self.query_processors {
            processor.dispatch(&aggregate_id, &committed_events);
        }
        Ok(())
    }

    fn duplicate(wrapped_events: &[MessageEnvelope<A, E>]) -> Vec<MessageEnvelope<A, E>> {
        let mut committed_events = Vec::new();
        for wrapped_event in wrapped_events {
            committed_events.push((*wrapped_event).clone());
        }
        committed_events
    }

    fn wrap_events(&self, aggregate_id: &str, current_sequence: usize, resultant_events: Vec<E>, base_metadata: HashMap<String, String>) -> Vec<MessageEnvelope<A, E>> {
        let mut sequence = current_sequence;
        let mut wrapped_events: Vec<MessageEnvelope<A, E>> = Vec::new();
        for payload in resultant_events {
            sequence += 1;
            let aggregate_type = A::aggregate_type().to_string();
            let aggregate_id: String = aggregate_id.to_string();
            let sequence = sequence;
            let metadata = base_metadata.clone();
            wrapped_events.push(MessageEnvelope {
                aggregate_id,
                sequence,
                aggregate_type,
                payload,
                metadata,
                _phantom: PhantomData,
            });
        }
        wrapped_events
    }

    fn load_aggregate(&self, aggregate_id: &str) -> (A, usize) {
        let committed_events = self.store.load(aggregate_id);
        let mut aggregate = A::default();
        let mut current_sequence = 0;
        for envelope in committed_events {
            current_sequence = envelope.sequence;
            let event = envelope.payload;
            event.apply(&mut aggregate);
        }
        (aggregate, current_sequence)
    }
}