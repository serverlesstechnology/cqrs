use crate::aggregate::{Aggregate, AggregateError};
use crate::event::{DomainEvent, EventEnvelope};

/// The abstract central source for loading past events and committing new events.
pub trait EventStore<A, E>
    where A: Aggregate,
          E: DomainEvent<A>
{
    /// Load all events for a particular `aggregate_id`
    fn load(&self, aggregate_id: &str) -> Vec<EventEnvelope<A, E>>;
    /// Commit new events
    fn commit(&self, events: Vec<EventEnvelope<A, E>>) -> Result<(), AggregateError>;
}
