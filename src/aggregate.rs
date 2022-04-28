use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::DomainEvent;

/// In CQRS (and Domain Driven Design) an `Aggregate` is the fundamental component that
/// encapsulates the state and application logic (aka business rules) for the application.
/// An `Aggregate` is always composed of a
/// [DDD entity](https://docs.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/microservice-domain-model#the-domain-entity-pattern)
/// along with all entities and
/// [value objects](https://docs.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/microservice-domain-model#the-value-object-pattern)
/// associated with it.
///
/// # Example of a 'Customer' aggregate
/// ```rust
/// # use cqrs_es::doc::{CustomerEvent, CustomerError, CustomerCommand, CustomerService};
/// # use cqrs_es::{Aggregate, AggregateError};
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
///     type Error = CustomerError;
///     type Services = CustomerService;
///
///
///     fn aggregate_type() -> String { "customer".to_string() }
///
///     async fn handle(&self, command: Self::Command, service: &Self::Services) -> Result<Vec<Self::Event>, Self::Error> {
///         match command {
///             CustomerCommand::AddCustomerName{name: changed_name} => {
///                 if self.name.is_some() {
///                     return Err("a name has already been added".into());
///                 }
///                 Ok(vec![CustomerEvent::NameAdded{name:changed_name}])
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
///             CustomerEvent::NameAdded{name: changed_name} => {
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
    type Command;
    /// Specifies the published events representing some change in state of the Aggregate.
    type Event: DomainEvent;
    /// The error returned when a command fails due to business logic.
    /// This is used to provide feedback to the user as to the nature of why the command was refused.
    type Error: std::error::Error;
    /// The external services required for the logic within the Aggregate
    type Services: Send + Sync;
    /// The aggregate type is used as the unique identifier for this aggregate and its events.
    /// This is used for persisting the events and snapshots to a database.
    fn aggregate_type() -> String;
    /// This method consumes and processes commands.
    /// The result should be either a vector of events if the command is successful,
    /// or an error if the command is rejected.
    ///
    /// _All business logic belongs in this method_.
    ///
    /// ```rust
    /// # use std::sync::Arc;
    /// use cqrs_es::{Aggregate, AggregateError};
    /// # use serde::{Serialize, Deserialize, de::DeserializeOwned};
    /// # use cqrs_es::doc::{CustomerCommand, CustomerError, CustomerEvent, CustomerService};
    /// # use async_trait::async_trait;
    /// #[derive(Default,Serialize,Deserialize)]
    /// # struct Customer {
    /// #     name: Option<String>,
    /// #     email: Option<String>,
    /// # }
    /// # #[async_trait]
    /// # impl Aggregate for Customer {
    /// #     type Command = CustomerCommand;
    /// #     type Event = CustomerEvent;
    /// #     type Error = CustomerError;
    /// #     type Services = CustomerService;
    /// #     fn aggregate_type() -> String { "customer".to_string() }
    /// async fn handle(&self, command: Self::Command, service: &Self::Services) -> Result<Vec<Self::Event>, Self::Error> {
    ///     match command {
    ///         CustomerCommand::AddCustomerName{name: changed_name} => {
    ///             if self.name.is_some() {
    ///                 return Err("a name has already been added".into());
    ///             }
    ///             Ok(vec![CustomerEvent::NameAdded{ name: changed_name}])
    ///         }
    ///
    ///         CustomerCommand::UpdateEmail { new_email } => {
    ///             Ok(vec![CustomerEvent::EmailUpdated { new_email }])
    ///         }
    ///     }
    /// }
    /// # fn apply(&mut self, event: Self::Event) {}
    /// # }
    /// ```
    async fn handle(
        &self,
        command: Self::Command,
        service: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error>;
    /// This is used to update the aggregate's state once an event has been committed.
    /// Any events returned from the `handle` method will be applied using this method
    /// in order to populate the state of the aggregate instance.
    ///
    /// The source of truth used in the CQRS framework determines when the events are
    /// applied to an aggregate:
    /// - event sourced - All events are applied every time the aggregate is loaded.
    /// - aggregate sourced - Events are applied immediately after they are returned from `handle`
    /// (and before they are committed) and the resulting aggregate instance is serialized and persisted.
    /// - snapshots - Uses a combination of the above patterns.
    ///
    /// _No business logic should be placed here_, this is only used for updating the aggregate state.
    ///
    /// ```rust
    /// # use std::sync::Arc;
    /// # use serde::{Serialize, Deserialize, de::DeserializeOwned};
    /// # use cqrs_es::doc::{CustomerCommand, CustomerError, CustomerEvent, CustomerService};
    /// use cqrs_es::{Aggregate, AggregateError};
    /// use async_trait::async_trait;
    /// #[derive(Default,Serialize,Deserialize)]
    /// # struct Customer {
    /// #     name: Option<String>,
    /// #     email: Option<String>,
    /// # }
    /// # #[async_trait]
    /// # impl Aggregate for Customer {
    /// #     type Command = CustomerCommand;
    /// #     type Event = CustomerEvent;
    /// #     type Error = CustomerError;
    /// #     type Services = CustomerService;
    /// #     fn aggregate_type() -> String { "customer".to_string() }
    /// # async fn handle(&self, command: Self::Command, service: &Self::Services) -> Result<Vec<Self::Event>, Self::Error> {
    /// # Ok(vec![])
    /// # }
    /// fn apply(&mut self, event: Self::Event) {
    ///     match event {
    ///         CustomerEvent::NameAdded{name} => {
    ///             self.name = Some(name);
    ///         }
    ///
    ///         CustomerEvent::EmailUpdated{new_email} => {
    ///             self.email = Some(new_email);
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    fn apply(&mut self, event: Self::Event);
}
