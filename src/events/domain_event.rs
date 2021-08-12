use serde::{
    de::DeserializeOwned,
    Serialize,
};
use std::fmt;

/// A `DomainEvent` represents any business change in the state of an
/// `Aggregate`. `DomainEvent`s are immutable and with [event sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)
/// they are the source of truth.
///
/// The name of a `DomainEvent` should always be in the past tense,
/// e.g.,
/// - `AdminPrivilegesGranted`
/// - `EmailAddressChanged`
/// - `DependencyAdded`
///
/// To simplify serialization, an event should be an enum, and each
/// element should have a payload. By convention, the payload has the
/// same name as the element, and elements that do not require
/// additional information use an empty payload.
///
/// Though the `DomainEvent` trait only has a single function, the
/// events must also derive a number of standard traits.
/// - `Clone` - events may be cloned throughout the framework,
///   particularly when applied to queries
/// - `Serialize` and `Deserialize` - required for persistence
/// - `PartialEq` and `Debug` - needed for effective testing
///
/// # Examples
/// ```
/// # use cqrs_es2::doc::Customer;
/// # use cqrs_es2::{Aggregate,DomainEvent};
/// # use serde::{Serialize,Deserialize};
/// #[derive(
///     Clone,
///     Debug,
///     Serialize,
///     Deserialize,
///     PartialEq
/// )]
/// pub enum CustomerEvent {
///     NameChanged(NameChanged),
///     EmailUpdated(EmailUpdated),
/// }
///
/// #[derive(
///     Clone,
///     Debug,
///     Serialize,
///     Deserialize,
///     PartialEq
/// )]
/// pub struct NameChanged {
///     changed_name: String,
/// }
///
/// #[derive(
///     Clone,
///     Debug,
///     Serialize,
///     Deserialize,
///     PartialEq
/// )]
/// pub struct EmailUpdated {
///     new_email: String,
/// }
/// ```
pub trait DomainEvent:
    Serialize
    + DeserializeOwned
    + Clone
    + PartialEq
    + fmt::Debug
    + Sync
    + Send {
}
