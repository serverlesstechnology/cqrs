use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{AggregateError, DomainEvent};

/// In CQRS (and Domain Driven Design) an `Aggregate` is the fundamental component that
/// encapsulates the state and application logic (aka business rules) for the application.
/// An `Aggregate` is always an entity along with all objects associated with it.
///
/// # Examples
/// ```rust
/// # use cqrs_es::doc::{CustomerEvent, CustomerCommand};
/// # use cqrs_es::{Aggregate, AggregateError, UserErrorPayload};
/// # use serde::{Serialize,Deserialize};
/// # use async_trait::async_trait;
/// #[derive(Default,Serialize,Deserialize)]
/// struct Customer {
///     name: Option<String>,
///     email: Option<String>,
/// }
///
/// #[async_trait]
/// impl Aggregate for Customer {
///     type Command = CustomerCommand;
///     type Event = CustomerEvent;
///     type Error = UserErrorPayload;
///
///     fn aggregate_type() -> String { "customer".to_string() }
///
///     async fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, AggregateError<UserErrorPayload>> {
///         match command {
///             CustomerCommand::AddCustomerName{changed_name} => {
///                 if self.name.is_some() {
///                     return Err("a name has already been added".into());
///                 }
///                 Ok(vec![CustomerEvent::NameAdded{changed_name}])
///             }
///
///             CustomerCommand::UpdateEmail { new_email } => {
///                 Ok(vec![CustomerEvent::EmailUpdated { new_email }])
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
#[async_trait]
pub trait Aggregate: Default + Serialize + DeserializeOwned + Sync + Send {
    /// Specifies the inbound command used to make changes in the state of the Aggregate.
    /// This is most easily accomplished with an enum;
    type Command;
    /// Specifies the published events representing some change in state of the Aggregate.
    /// This is most easily accomplished with an enum;
    type Event: DomainEvent;
    /// The error returned when a command fails due to business logic.
    /// Usually used to provide feedback to the user as to the nature of why the command was refused.
    type Error: std::error::Error;
    /// The aggregate type is used as the identifier for this aggregate and its events upon
    /// serialization. The value returned should be unique.
    fn aggregate_type() -> String;
    /// This method consumes and processes the fired command. During operation the aggregate
    /// will be populated with the current state.
    ///
    /// The result is either a vector of events to be committed or an error if the command is
    /// rejected.
    ///
    /// *All business logic should be placed here*.
    ///
    /// ```ignore
    /// # use cqrs_es::{AggregateError,UserErrorPayload};
    /// # use cqrs_es::doc::{CustomerCommand,CustomerEvent};
    /// async fn handle(&self, command: CustomerCommand) -> Result<Vec<CustomerEvent>, AggregateError<UserErrorPayload>> {
    ///     match command {
    ///         CustomerCommand::AddCustomerName{changed_name} => {
    ///             if self.name.is_some() {
    ///                 return Err("a name has already been added".into());
    ///             }
    ///             Ok(vec![CustomerEvent::NameAdded{changed_name}])
    ///         }
    ///
    ///         CustomerCommand::UpdateEmail { new_email } => {
    ///             Ok(vec![CustomerEvent::EmailUpdated { new_email }])
    ///         }
    ///     }
    /// }
    /// ```
    async fn handle(
        &self,
        command: Self::Command,
    ) -> Result<Vec<Self::Event>, AggregateError<Self::Error>>;
    /// This is used to update the aggregate's state once an event has been committed.
    /// When event sourcing is used all previous events are loaded and applied (using this method)
    /// in order to populate the state of the aggregate instance.
    ///
    /// *No business logic should be placed here*, this is only used for updating the aggregate state.
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
