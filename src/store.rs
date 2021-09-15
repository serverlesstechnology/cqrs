use std::collections::HashMap;

use crate::aggregate::{Aggregate, AggregateError};
use crate::event::{EventEnvelope};

/// The abstract central source for loading past events and committing new events.
pub trait EventStore<A, AC> : Send + Sync
    where A: Aggregate,
          AC: AggregateContext<A>
{
    /// Load all events for a particular `aggregate_id`
    fn load(&self, aggregate_id: &str) -> Vec<EventEnvelope<A>>;
    /// Load aggregate at current state
    fn load_aggregate(&self, aggregate_id: &str) -> AC;
    /// Commit new events
    fn commit(&self, events: Vec<A::Event>, context: AC, metadata: HashMap<String, String>) -> Result<Vec<EventEnvelope<A>>, AggregateError>;

    /// Method to wrap a set of events with the additional metadata needed for persistence and publishing
    fn wrap_events(&self, aggregate_id: &str, current_sequence: usize, resultant_events: Vec<A::Event>, base_metadata: HashMap<String, String>) -> Vec<EventEnvelope<A>> {
        let mut sequence = current_sequence;
        let mut wrapped_events: Vec<EventEnvelope<A>> = Vec::new();
        for payload in resultant_events {
            sequence += 1;
            let aggregate_type = A::aggregate_type().to_string();
            let aggregate_id: String = aggregate_id.to_string();
            let sequence = sequence;
            let metadata = base_metadata.clone();
            wrapped_events.push(EventEnvelope::new_with_metadata(
                aggregate_id,
                sequence,
                aggregate_type,
                payload,
                metadata,
            ));
        }
        wrapped_events
    }
}

/// Returns the aggregate and context around it that is needed when committing events
pub trait AggregateContext<A>
    where A: Aggregate {
    /// The aggregate instance with all state loaded.
    fn aggregate(&self) -> &A;
}
