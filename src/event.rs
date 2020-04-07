use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::aggregate::Aggregate;

/// A `DomainEvent` represents any business change in the state of an `Aggregate`. `DomainEvent`s
/// are immutable and with [event sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)
/// they are the source of truth.
pub trait DomainEvent<A: Aggregate>: Serialize + DeserializeOwned + Clone + PartialEq + fmt::Debug {
    /// apply encapsulates all of the logic that determines how an event modifies the state of an
    /// `Aggregate`.
    ///
    /// Note the lack of return value, because they will be replay events can not change the state
    /// of any other object and should never result in an error.
    fn apply(self, aggregate: &mut A);
}

/// `MessageEnvelope` encapsulates an event with pertinent information. All of the associated data
/// will be transported and persisted together.
#[derive(Debug)]
pub struct MessageEnvelope<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    pub aggregate_id: String,
    pub sequence: usize,
    pub aggregate_type: String,
    pub payload: E,
    pub metadata: HashMap<String, String>,
    pub _phantom: PhantomData<A>,
}

impl<A,E> Clone for MessageEnvelope<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    fn clone(&self) -> Self {
        MessageEnvelope {
            aggregate_id: self.aggregate_id.clone(),
            sequence: self.sequence,
            aggregate_type: self.aggregate_type.clone(),
            payload: self.payload.clone(),
            metadata: self.metadata.clone(),
            _phantom: PhantomData
        }
    }
}

impl<A, E> MessageEnvelope<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    pub fn new(aggregate_id: String, sequence: usize, aggregate_type: String, payload: E) -> Self
    {
        MessageEnvelope {
            aggregate_id,
            sequence,
            aggregate_type,
            payload,
            metadata: Default::default(),
            _phantom: PhantomData,
        }
    }
    pub fn new_with_metadata(aggregate_id: String, sequence: usize, aggregate_type: String, payload: E, metadata: HashMap<String, String>) -> Self
    {
        MessageEnvelope {
            aggregate_id,
            sequence,
            aggregate_type,
            payload,
            metadata,
            _phantom: PhantomData,
        }
    }
}
