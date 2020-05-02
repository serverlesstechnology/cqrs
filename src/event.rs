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
/// The name of a `DomainEvent` should always be in the past tense, e.g.,
/// - `AdminPrivilegesGranted`
/// - `EmailAddressChanged`
/// - `DependencyAdded`
///
/// To simplify serialization, an event should be an enum, and each element should have a payload.
/// By convention, the payload has the same name as the element, and elements that do not require
/// additional information use an empty payload.
///
/// Though the `DomainEvent` trait only has a single function, the events must also derive a
/// number of standard traits.
/// - `Clone` - events may be cloned throughout the framework, particularly when applied to queries
/// - `Serialize` and `Deserialize` - required for persistence
/// - `PartialEq` and `Debug` - needed for effective testing
///
/// # Examples
/// ```
/// # use cqrs_es::doc::Customer;
/// # use cqrs_es::{Aggregate,DomainEvent};
/// # use serde::{Serialize,Deserialize};
/// #[derive(Clone,Debug,Serialize,Deserialize,PartialEq)]
/// pub enum CustomerEvent {
///     NameChanged(NameChanged),
///     EmailUpdated(EmailUpdated)
/// }
///
/// #[derive(Clone,Debug,Serialize,Deserialize,PartialEq)]
/// pub struct NameChanged {
///     changed_name: String
/// }
///
/// #[derive(Clone,Debug,Serialize,Deserialize,PartialEq)]
/// pub struct EmailUpdated {
///     new_email: String
/// }
/// ```
pub trait DomainEvent<A: Aggregate>: Serialize + DeserializeOwned + Clone + PartialEq + fmt::Debug + Sync + Send {
    /// This method encapsulates all of the logic that determines how an event modifies the state of an
    /// `Aggregate`.
    ///
    /// Note the lack of return value. Events are records of past changes and so should never
    /// result in an error.
    ///
    /// # Examples
    /// ```
    /// # use cqrs_es::doc::{Customer, NameAdded, EmailUpdated};
    /// # use cqrs_es::{Aggregate,DomainEvent};
    /// # use serde::{Serialize,Deserialize};
    /// # #[derive(Clone,Debug,Serialize,Deserialize,PartialEq)]
    /// # pub enum CustomerEvent {
    /// #     NameAdded(NameAdded),
    /// #     EmailUpdated(EmailUpdated)
    /// # }
    /// impl DomainEvent<Customer> for CustomerEvent {
    ///     fn apply(self, customer: &mut Customer) {
    ///         match self {
    ///             CustomerEvent::NameAdded(payload) => {
    ///                 customer.name = payload.changed_name;
    ///             }
    ///             CustomerEvent::EmailUpdated(payload) => {
    ///                 customer.email = payload.new_email;
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    fn apply(self, aggregate: &mut A);
}

/// `EventEnvelope` is a data structure that encapsulates an event with along with it's pertinent
/// information. All of the associated data will be transported and persisted together.
///
/// Within any system an event must be unique based on its' `aggregate_type`, `aggregate_id` and
/// `sequence`.
#[derive(Debug)]
pub struct EventEnvelope<A, E>
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

impl<A,E> Clone for EventEnvelope<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    fn clone(&self) -> Self {
        EventEnvelope {
            aggregate_id: self.aggregate_id.clone(),
            sequence: self.sequence,
            aggregate_type: self.aggregate_type.clone(),
            payload: self.payload.clone(),
            metadata: self.metadata.clone(),
            _phantom: PhantomData
        }
    }
}

impl<A, E> EventEnvelope<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{

    /// A convenience function for packaging an event in an `EventEnvelope`, used for
    /// testing `QueryProcessor`s.
    pub fn new(aggregate_id: String, sequence: usize, aggregate_type: String, payload: E) -> Self
    {
        EventEnvelope {
            aggregate_id,
            sequence,
            aggregate_type,
            payload,
            metadata: Default::default(),
            _phantom: PhantomData,
        }
    }
    /// A convenience function for packaging an event in an `EventEnvelope`, used for
    /// testing `QueryProcessor`s. This version allows custom metadata to also be processed.
    pub fn new_with_metadata(aggregate_id: String, sequence: usize, aggregate_type: String, payload: E, metadata: HashMap<String, String>) -> Self
    {
        EventEnvelope {
            aggregate_id,
            sequence,
            aggregate_type,
            payload,
            metadata,
            _phantom: PhantomData,
        }
    }
}
