use std::collections::HashMap;

use cqrs_es2_core::{
    AggregateContext,
    Error,
    EventContext,
    IAggregate,
    ICommand,
    IEvent,
};

/// The abstract central source for loading past events and committing
/// new events.
pub trait IEventStore<C: ICommand, E: IEvent, A: IAggregate<C, E>> {
    /// Load all events for a particular `aggregate_id`
    fn load_events(
        &mut self,
        aggregate_id: &str,
        with_metadata: bool,
    ) -> Result<Vec<EventContext<C, E>>, Error>;

    /// Load aggregate at current state
    fn load_aggregate(
        &mut self,
        aggregate_id: &str,
    ) -> Result<AggregateContext<C, E, A>, Error>;

    /// Commit new events
    fn commit(
        &mut self,
        events: Vec<E>,
        context: AggregateContext<C, E, A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventContext<C, E>>, Error>;

    /// Method to wrap a set of events with the additional metadata
    /// needed for persistence and publishing
    fn wrap_events(
        &self,
        aggregate_id: &str,
        current_sequence: usize,
        events: Vec<E>,
        metadata: HashMap<String, String>,
    ) -> Vec<EventContext<C, E>> {
        let mut sequence = current_sequence;

        let mut wrapped_events = Vec::new();

        for payload in events {
            sequence += 1;

            wrapped_events.push(EventContext::new(
                aggregate_id.to_string(),
                sequence,
                payload,
                metadata.clone(),
            ));
        }

        wrapped_events
    }
}
