use std::collections::HashMap;

use crate::{
    aggregates::{
        AggregateContext,
        IAggregate,
    },
    errors::AggregateError,
    events::EventContext,
};

/// The abstract central source for loading past events and committing
/// new events.
pub trait IEventStore<A: IAggregate> {
    /// Load all events for a particular `aggregate_id`
    fn load_events(
        &mut self,
        aggregate_id: &str,
        with_metadata: bool,
    ) -> Result<Vec<EventContext<A>>, AggregateError>;

    /// Load aggregate at current state
    fn load_aggregate(
        &mut self,
        aggregate_id: &str,
    ) -> Result<AggregateContext<A>, AggregateError>;

    /// Commit new events
    fn commit(
        &mut self,
        events: Vec<A::Event>,
        context: AggregateContext<A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventContext<A>>, AggregateError>;

    /// Method to wrap a set of events with the additional metadata
    /// needed for persistence and publishing
    fn wrap_events(
        &self,
        aggregate_id: &str,
        current_sequence: usize,
        resultant_events: Vec<A::Event>,
        base_metadata: HashMap<String, String>,
    ) -> Vec<EventContext<A>> {
        let mut sequence = current_sequence;

        let mut wrapped_events: Vec<EventContext<A>> = Vec::new();

        for payload in resultant_events {
            sequence += 1;

            wrapped_events.push(EventContext {
                aggregate_id: aggregate_id.to_string(),
                sequence,
                payload,
                metadata: base_metadata.clone(),
            });
        }

        wrapped_events
    }
}
