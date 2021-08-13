use serde::{
    de::DeserializeOwned,
    Serialize,
};

use crate::{
    commands::DomainCommand,
    events::DomainEvent,
};

use super::errors::AggregateError;

/// In CQRS (and Domain Driven Design) an `Aggregate` is the
/// fundamental component that encapsulates the state and application
/// logic (aka business rules) for the application. An `Aggregate` is
/// always an entity along with all objects associated with it.
///
/// # Examples
/// ```rust
/// # use cqrs_es2::doc::{CustomerEvent, CustomerCommand, NameAdded};
/// # use cqrs_es2::{Aggregate, AggregateError};
/// # use serde::{Serialize,Deserialize};
/// #[derive(Serialize,Deserialize)]
/// struct Customer {
///     customer_id: String,
///     name: String,
///     email: String,
/// }
///
/// impl Aggregate for Customer {
///     type Command = CustomerCommand;
///     type Event = CustomerEvent;
///
///     fn aggregate_type() -> &'static str { "customer" }
///
///     fn handle(&self, command: &Self::Command) -> Result<Vec<Self::Event>, AggregateError> {
///         match command {
///             CustomerCommand::AddCustomerName(payload) => {
///                 if self.name.as_str() != "" {
///                     return Err(AggregateError::new("a name has already been added for this customer"));
///                 }
///                 let payload = NameAdded {
///                     changed_name: payload.changed_name.clone()
///                 };
///                 Ok(vec![CustomerEvent::NameAdded(payload)])
///             }
///             CustomerCommand::UpdateEmail(_) => {
///                 Ok(Default::default())
///             }
///         }
///     }
///
///     fn apply(&mut self, event: &Self::Event) {
///         match event {
///             CustomerEvent::NameAdded(payload) => {
///                 self.name = payload.changed_name.clone();
///             }
///             CustomerEvent::EmailUpdated(payload) => {
///                 self.email = payload.new_email.clone();
///             }
///         }
///     }
/// }
///
/// impl Default for Customer {fn default() -> Self {
///         Customer {
///             customer_id: "".to_string(),
///             name: "".to_string(),
///             email: "".to_string(),
///         }
///     }
/// }
/// ```
pub trait Aggregate:
    Default + Serialize + DeserializeOwned + Sync + Send {
    /// An inbound command used to make changes in the state of the
    /// Aggregate
    type Command: DomainCommand;

    /// An event representing some change in state of the Aggregate
    type Event: DomainEvent;

    /// aggregate_type is a unique identifier for this aggregate
    fn aggregate_type() -> &'static str;

    /// handle inbound command and return a vector of events or an
    /// error
    fn handle(
        &self,
        command: &Self::Command,
    ) -> Result<Vec<Self::Event>, AggregateError>;

    /// Update the aggregate's state with an event
    fn apply(
        &mut self,
        event: &Self::Event,
    );
}
