use async_trait::async_trait;
use std::collections::HashMap;

use crate::aggregate::Aggregate;
use crate::event::EventEnvelope;
use crate::AggregateError;

/// The abstract central source for loading past events and committing new events.
#[async_trait]
pub trait EventStore<A>: Send + Sync
where
    A: Aggregate,
{
    /// Provides the current state of an aggregate along with surrounding context.
    /// This is used by the [CqrsFramework](struct.CqrsFramework.html) when loading
    /// an aggregate in order to handle incoming commands.
    type AC: AggregateContext<A>;

    /// Load all events for a particular `aggregate_id`
    async fn load(&self, aggregate_id: &str) -> Vec<EventEnvelope<A>>;
    /// Load aggregate at current state
    async fn load_aggregate(&self, aggregate_id: &str) -> Self::AC;
    /// Commit new events
    async fn commit(
        &self,
        events: Vec<A::Event>,
        context: Self::AC,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError>;

    /// Method to wrap a set of events with the additional metadata needed for persistence and publishing
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
where
    A: Aggregate,
{
    /// The aggregate instance with all state loaded.
    fn aggregate(&self) -> &A;
}
