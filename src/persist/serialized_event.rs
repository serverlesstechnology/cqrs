use std::convert::TryFrom;

use crate::{Aggregate, DomainEvent, EventEnvelope};
use serde_json::Value;

use crate::persist::{EventStoreAggregateContext, EventUpcaster, PersistenceError};

/// A serialized version of an event with metadata.
/// Used by repositories to store and load events from a database.
#[derive(Clone, Debug, PartialEq)]
pub struct SerializedEvent {
    /// The id of the aggregate instance.
    pub aggregate_id: String,
    /// The sequence number of the event for this aggregate instance.
    pub sequence: usize,
    /// The type of aggregate the event applies to.
    pub aggregate_type: String,
    /// The type of event that is serialized.
    pub event_type: String,
    /// The version of event that is serialized.
    pub event_version: String,
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
        event_type: String,
        event_version: String,
        payload: Value,
        metadata: Value,
    ) -> Self {
        Self {
            aggregate_id,
            sequence,
            aggregate_type,
            event_type,
            event_version,
            payload,
            metadata,
        }
    }
}

pub(crate) fn serialize_events<A: Aggregate>(
    events: &[EventEnvelope<A>],
) -> Result<Vec<SerializedEvent>, PersistenceError> {
    let mut result: Vec<SerializedEvent> = Default::default();
    for event in events {
        result.push(SerializedEvent::try_from(event)?);
    }
    Ok(result)
}

pub(crate) fn deserialize_events<A: Aggregate>(
    events: Vec<SerializedEvent>,
    upcasters: &Option<Vec<Box<dyn EventUpcaster>>>,
) -> Result<Vec<EventEnvelope<A>>, PersistenceError> {
    let mut result: Vec<EventEnvelope<A>> = Default::default();
    for event in events {
        let upcasted_event = match upcasters {
            None => event,
            Some(upcasters) => {
                let mut upcasted_event = event;
                for upcaster in upcasters {
                    if upcaster
                        .can_upcast(&upcasted_event.event_type, &upcasted_event.event_version)
                    {
                        upcasted_event = upcaster.upcast(upcasted_event)
                    }
                }
                upcasted_event
            }
        };
        result.push(EventEnvelope::<A>::try_from(upcasted_event)?);
    }
    Ok(result)
}

impl<A: Aggregate> TryFrom<&EventEnvelope<A>> for SerializedEvent {
    type Error = PersistenceError;

    fn try_from(event: &EventEnvelope<A>) -> Result<Self, Self::Error> {
        let aggregate_type = A::aggregate_type();
        let event_type = event.payload.event_type();
        let event_version = event.payload.event_version();
        let payload = serde_json::to_value(&event.payload)?;
        let metadata = serde_json::to_value(&event.metadata)?;
        Ok(Self {
            aggregate_id: event.aggregate_id.clone(),
            sequence: event.sequence,
            aggregate_type,
            event_type,
            event_version,
            payload,
            metadata,
        })
    }
}

/// A serialized version of a snapshot.
/// Used by repositories to store and load snapshots from a database.
#[derive(Debug, PartialEq)]
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
