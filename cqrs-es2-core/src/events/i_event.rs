use serde::{
    de::DeserializeOwned,
    Serialize,
};
use std::fmt::Debug;

/// An `IEvent` represents any business change in the state of an
/// `Aggregate`. `IEvent`s are immutable and with
/// [event sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)
/// they are the source of truth.
///
/// The name of an `IEvent` should always be in the past tense,
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
/// Though the `IEvent` trait only has a single function, the
/// events must also derive a number of standard traits.
/// - `Clone` - events may be cloned throughout the framework,
///   particularly when applied to queries
/// - `Serialize` and `Deserialize` - required for persistence
/// - `PartialEq` and `Debug` - needed for effective testing
///
/// # Examples
/// ```rust
/// use serde::{
///     Deserialize,
///     Serialize,
/// };
/// use std::fmt::Debug;
///
/// use cqrs_es2_core::IEvent;
///
/// #[derive(
///     Debug,
///     PartialEq,
///     Clone,
///     Serialize,
///     Deserialize
/// )]
/// pub enum CustomerEvent {
///     NameAdded(NameAdded),
///     EmailUpdated(EmailUpdated),
/// }
///
/// #[derive(
///     Debug,
///     PartialEq,
///     Clone,
///     Serialize,
///     Deserialize
/// )]
/// pub struct NameAdded {
///     changed_name: String,
/// }
///
/// #[derive(
///     Debug,
///     PartialEq,
///     Clone,
///     Serialize,
///     Deserialize
/// )]
/// pub struct EmailUpdated {
///     new_email: String,
/// }
///
/// impl IEvent for CustomerEvent {};
/// ```
pub trait IEvent:
    Debug + PartialEq + Clone + Serialize + DeserializeOwned + Sync + Send
{
}
