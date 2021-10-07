use serde::de::DeserializeOwned;
use serde::{Serialize};

use crate::{AggregateError, DomainEvent};

/// In CQRS (and Domain Driven Design) an `Aggregate` is the fundamental component that
/// encapsulates the state and application logic (aka business rules) for the application.
/// An `Aggregate` is always an entity along with all objects associated with it.
///
/// # Examples
/// ```rust
/// # use cqrs_es::doc::{CustomerEvent, CustomerCommand};
/// # use cqrs_es::{Aggregate, AggregateError};
/// # use serde::{Serialize,Deserialize};
/// #[derive(Default,Serialize,Deserialize)]
/// struct Customer {
///     name: Option<String>,
///     email: Option<String>,
/// }
///
/// impl Aggregate for Customer {
///     type Command = CustomerCommand;
///     type Event = CustomerEvent;
///
///     fn aggregate_type() -> &'static str { "customer" }
///
///     fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, AggregateError> {
///         match command {
///             CustomerCommand::AddCustomerName{changed_name} => {
///                 if self.name.is_some() {
///                     return Err(AggregateError::new("a name has already been added"));
///                 }
///                 Ok(vec![CustomerEvent::NameAdded{changed_name}])
///             }
///
///             CustomerCommand::UpdateEmail{..} => {
///                 Ok(Default::default())
///             }
///         }
///     }
///
///     fn apply(&mut self, event: Self::Event) {
///         match event {
///             CustomerEvent::NameAdded{changed_name} => {
///                 self.name = Some(changed_name);
///             }
///
///             CustomerEvent::EmailUpdated{new_email} => {
///                 self.email = Some(new_email);
///             }
///         }
///     }
/// }
/// ```
pub trait Aggregate: Default + Serialize + DeserializeOwned + Sync + Send {
    /// Specifies the inbound command used to make changes in the state of the Aggregate.
    /// This is most easily accomplished with an enum;
    type Command;
    /// Specifies the published events representing some change in state of the Aggregate.
    /// This is most easily accomplished with an enum;
    type Event: DomainEvent;
    /// The aggregate type is used as the identifier for this aggregate and its events upon
    /// serialization. The value returned should be unique.
    fn aggregate_type() -> &'static str;
    /// This method consumes and processes the fired command. During operation the aggregate
    /// will be populated with the current state.
    ///
    /// The result is either a vector of events to be committed or an error if the command is
    /// rejected.
    ///
    /// *All business logic should be placed here*.
    ///
    /// ```ignore
    /// fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, AggregateError> {
    ///     match command {
    ///         CustomerCommand::AddCustomerName{changed_name} => {
    ///             if self.name.is_some() {
    ///                 return Err(AggregateError::new("a name has already been added"));
    ///             }
    ///             Ok(vec![CustomerEvent::NameAdded{changed_name}])
    ///         }
    ///
    ///         CustomerCommand::UpdateEmail{..} => {
    ///             Ok(Default::default())
    ///         }
    ///     }
    /// }
    /// ```
    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, AggregateError>;
    /// This is used to update the aggregate's state once an event has been committed.
    /// When event sourcing is used all previous events are loaded and applied (using this method)
    /// in order to populate the state of the aggregate instance.
    ///
    /// *No business logic should be placed here*, this is only for updating state.
    ///
    /// ```ignore
    /// fn apply(&mut self, event: Self::Event) {
    ///     match event {
    ///         CustomerEvent::NameAdded{changed_name} => {
    ///             self.name = Some(changed_name);
    ///         }
    ///
    ///         CustomerEvent::EmailUpdated{new_email} => {
    ///             self.email = Some(new_email);
    ///         }
    ///     }
    /// }
    /// ```
    fn apply(&mut self, event: Self::Event);
}