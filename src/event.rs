use std::collections::HashMap;
use std::fmt;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::aggregate::Aggregate;

/// A `DomainEvent` represents any business change in the state of an `Aggregate`. `DomainEvent`s
/// are immutable, and when
/// [event sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)
/// is used they are the single source of truth.
///
/// The name of a `DomainEvent` should always be in the past tense, e.g.,
/// - AdminPrivilegesGranted
/// - EmailAddressChanged
/// - DependencyAdded
///
/// To simplify serialization, an event should be an enum, and each variant should carry any
/// important information.
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
///     NameChanged{ changed_name: String },
///     EmailUpdated{ new_email: String },
/// }
/// ```
pub trait DomainEvent:
    Serialize + DeserializeOwned + Clone + PartialEq + fmt::Debug + Sync + Send
{
    /// A name specifying the event, used for event upcasting.
    fn event_type(&self) -> String;
    /// A version of the `event_type`, used for event upcasting.
    fn event_version(&self) -> String;
}

/// `EventEnvelope` is a data structure that encapsulates an event with its pertinent
/// information.
/// All of the associated data will be transported and persisted together and will be available
/// for queries.
///
/// Within any system an event must be unique based on the compound key composed of its:
/// - [`aggregate_type`](https://docs.rs/cqrs-es/latest/cqrs_es/trait.Aggregate.html#tymethod.aggregate_type)
/// - `aggregate_id`
/// - `sequence`
///
/// Thus an `EventEnvelope` provides a uniqueness value along with an event `payload` and
/// `metadata`.
#[derive(Debug)]
pub struct EventEnvelope<A>
where
    A: Aggregate,
{
    /// The id of the aggregate instance.
    pub aggregate_id: String,
    /// The sequence number for an aggregate instance.
    pub sequence: usize,
    /// The event payload with all business information.
    pub payload: A::Event,
    /// Additional metadata for use in auditing, logging or debugging purposes.
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
