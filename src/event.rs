use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::aggregate::Aggregate;

/// A `DomainEvent` represents any business change in the state of an `Aggregate`. `DomainEvent`s
/// are immutable and with [event sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)
/// they are the source of truth.
///
/// A `DomainEvent` should always be in the past tense, e.g.,
/// - `AdminPrivilegesGranted`
/// - `EmailAddressChanged`
/// - `DependencyAdded`
pub trait DomainEvent<A: Aggregate>: Serialize + DeserializeOwned + Clone + PartialEq + fmt::Debug {
    /// apply encapsulates all of the logic that determines how an event modifies the state of an
    /// `Aggregate`.
    ///
    /// Note the lack of return value, because they will be replay events can not change the state
    /// of any other object and should never result in an error.
    fn apply(self, aggregate: &mut A);
}

/// `MessageEnvelope` is a data structure that encapsulates an event with along with it's pertinent
/// information. All of the associated data will be transported and persisted together.
///
/// Within any system an event must be unique based on its' `aggregate_type`, `aggregate_id` and
/// `sequence`.
#[derive(Debug)]
pub struct MessageEnvelope<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    /// The id of the aggregate instance.
    pub aggregate_id: String,
    /// The sequence number for an aggregate instance.
    pub sequence: usize,
    /// The type of aggregate the event applies to.
    pub aggregate_type: String,
    /// The event payload with all business information.
    pub payload: E,
    /// Additional metadata for use in auditing, logging or debugging purposes.
    pub metadata: HashMap<String, String>,
    pub(crate) _phantom: PhantomData<A>,
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

    /// Convenience function for packaging an event in a `MessageEnvelope`, used for
    /// testing `ViewProcessor`s.
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
    /// Convenience function for packaging an event in a `MessageEnvelope`, used for
    /// testing `ViewProcessor`s.
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
