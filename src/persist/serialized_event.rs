use std::convert::TryFrom;

use crate::{Aggregate, EventEnvelope};
use serde_json::Value;

use crate::persist::{EventStoreAggregateContext, PersistenceError};

/// A serialized version of an event with metadata.
/// Used by repositories to store and load events from a database.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SerializedEvent {
    /// The id of the aggregate instance.
    pub aggregate_id: String,
    /// The sequence number of the event for this aggregate instance.
    pub sequence: usize,
    /// The type of aggregate the event applies to.
    pub aggregate_type: String,
    /// The serialized domain event.
    pub payload: Value,
    /// Additional metadata, serialized from a HashMap<String,String>.
    pub metadata: Value,
}

impl SerializedEvent {
    /// Create a new SerializedEvent with the given values.
    pub fn new(
        aggregate_id: String,
        sequence: usize,
        aggregate_type: String,
        payload: Value,
        metadata: Value,
    ) -> Self {
        Self {
            aggregate_id,
            sequence,
            aggregate_type,
            payload,
            metadata,
        }
    }
}

impl<A: Aggregate> TryFrom<&EventEnvelope<A>> for SerializedEvent {
    type Error = PersistenceError;

    fn try_from(event: &EventEnvelope<A>) -> Result<Self, Self::Error> {
        let aggregate_type = A::aggregate_type();
        let payload = serde_json::to_value(&event.payload)?;
        let metadata = serde_json::to_value(&event.metadata)?;
        Ok(Self {
            aggregate_id: event.aggregate_id.clone(),
            sequence: event.sequence,
            aggregate_type,
            payload,
            metadata,
        })
    }
}

/// A serialized version of a snapshot.
/// Used by repositories to store and load snapshots from a database.
#[derive(Debug, PartialEq, Eq)]
pub struct SerializedSnapshot {
    /// The aggregate ID of the aggregate instance that has been loaded.
    pub aggregate_id: String,
    /// The current state of the aggregate instance.
    pub aggregate: Value,
    /// The last committed event sequence number for this aggregate instance.
    pub current_sequence: usize,
    /// The last committed snapshot version for this aggregate instance.
    pub current_snapshot: usize,
}

impl<A: Aggregate> TryFrom<SerializedSnapshot> for EventStoreAggregateContext<A> {
    type Error = PersistenceError;

    fn try_from(snapshot: SerializedSnapshot) -> Result<Self, Self::Error> {
        let aggregate = serde_json::from_value(snapshot.aggregate.clone())?;
        Ok(Self {
            aggregate_id: snapshot.aggregate_id,
            aggregate,
            current_sequence: snapshot.current_sequence,
            current_snapshot: Some(snapshot.current_snapshot),
        })
    }
}

impl<A: Aggregate> TryFrom<SerializedEvent> for EventEnvelope<A> {
    type Error = PersistenceError;

    fn try_from(event: SerializedEvent) -> Result<Self, Self::Error> {
        let payload = serde_json::from_value(event.payload)?;
        let metadata = serde_json::from_value(event.metadata)?;
        Ok(Self {
            aggregate_id: event.aggregate_id,
            sequence: event.sequence,
            payload,
            metadata,
        })
    }
}
