use std::collections::HashMap;

use crate::{
    aggregates::{
        AggregateContext,
        IAggregate,
    },
    errors::AggregateError,
    events::EventEnvelope,
};

/// The abstract central source for loading past events and committing
/// new events.
pub trait IEventStore<A: IAggregate> {
    /// Load all events for a particular `aggregate_id`
    fn load(
        &mut self,
        aggregate_id: &str,
    ) -> Vec<EventEnvelope<A>>;

    /// Load aggregate at current state
    fn load_aggregate(
        &mut self,
        aggregate_id: &str,
    ) -> AggregateContext<A>;

    /// Commit new events
    fn commit(
        &mut self,
        events: Vec<A::Event>,
        context: AggregateContext<A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError>;

    /// Method to wrap a set of events with the additional metadata
    /// needed for persistence and publishing
    fn wrap_events(
        &self,
        aggregate_id: &str,
        current_sequence: usize,
        resultant_events: Vec<A::Event>,
        base_metadata: HashMap<String, String>,
    ) -> Vec<EventEnvelope<A>> {
        let mut sequence = current_sequence;
        let mut wrapped_events: Vec<EventEnvelope<A>> = Vec::new();
        for payload in resultant_events {
            sequence += 1;
            let aggregate_id: String = aggregate_id.to_string();
            let sequence = sequence;
            let metadata = base_metadata.clone();
            wrapped_events.push(EventEnvelope::new_with_metadata(
                aggregate_id,
                sequence,
                payload,
                metadata,
            ));
        }
        wrapped_events
    }
}
