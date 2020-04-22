use crate::aggregate::{Aggregate, AggregateError};
use crate::event::{DomainEvent, MessageEnvelope};

/// The abstract central source for loading past events and committing new events.
pub trait EventStore<A, E>
    where A: Aggregate,
          E: DomainEvent<A>
{
    /// Load all events for a particular `aggregate_id`
    fn load(&self, aggregate_id: &str) -> Vec<MessageEnvelope<A, E>>;
    /// Commit new events
    fn commit(&self, events: Vec<MessageEnvelope<A, E>>) -> Result<(), AggregateError>;
}
