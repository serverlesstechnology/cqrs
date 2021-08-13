use std::collections::HashMap;

use crate::aggregates::Aggregate;

/// `EventEnvelope` is a data structure that encapsulates an event
/// with along with it's pertinent information. All of the associated
/// data will be transported and persisted together.
///
/// Within any system an event must be unique based on its'
/// `aggregate_type`, `aggregate_id` and `sequence`.
#[derive(Debug)]
pub struct EventEnvelope<A>
where
    A: Aggregate, {
    /// The id of the aggregate instance.
    pub aggregate_id: String,
    /// The sequence number for an aggregate instance.
    pub sequence: usize,
    /// The event payload with all business information.
    pub payload: A::Event,
    /// Additional metadata for use in auditing, logging or debugging
    /// purposes.
    pub metadata: HashMap<String, String>,
}

impl<A: Aggregate> Clone for EventEnvelope<A> {
    fn clone(&self) -> Self {
        EventEnvelope {
            aggregate_id: self.aggregate_id.clone(),
            sequence: self.sequence,
            payload: self.payload.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl<A: Aggregate> EventEnvelope<A> {
    /// A convenience function for packaging an event in an
    /// `EventEnvelope`, used for testing `QueryProcessor`s.
    pub fn new(
        aggregate_id: String,
        sequence: usize,
        payload: A::Event,
    ) -> Self {
        EventEnvelope {
            aggregate_id,
            sequence,
            payload,
            metadata: Default::default(),
        }
    }
    /// A convenience function for packaging an event in an
    /// `EventEnvelope`, used for testing `QueryProcessor`s. This
    /// version allows custom metadata to also be processed.
    pub fn new_with_metadata(
        aggregate_id: String,
        sequence: usize,
        payload: A::Event,
        metadata: HashMap<String, String>,
    ) -> Self {
        EventEnvelope {
            aggregate_id,
            sequence,
            payload,
            metadata,
        }
    }
}
