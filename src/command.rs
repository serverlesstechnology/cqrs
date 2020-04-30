use crate::aggregate::{Aggregate, AggregateError};
use crate::event::DomainEvent;

/// A `Command` represents a request to modify the state of an `Aggregate` by producing
/// `DomainEvent`s.
///
/// A `Command` is always named in the imperative, e.g.,
/// - `GrantAdminPrivileges`
/// - `ChangeEmailAddress`
/// - `AddDependency`
///
/// In it's simplest form a `Command` is a simple data object (possibly with no data if it
/// represents a boolean change to the system), but usually they are implemented with the derived
/// `Serialize` and `Deserialize` traits for use in a Restful endpoint.
///
/// # Examples
/// ```
/// pub struct ChangeName {
///     pub changed_name: String
/// }
/// ```
pub trait Command<A, E>
    where A: Aggregate,
          E: DomainEvent<A>
{
    /// The user should implement all business logic within the `handle` method of a `Command`.
    /// As input the current state of the aggregate is provided, from which a list of
    /// `DomainEvent`s is returned if the command is accepted.
    ///
    /// If the command is rejected an `AggregateError` (`AggregateError::UserError` in nearly
    /// all instances) should carry a message for the user as to why the request was rejected.
    ///
    /// # Error
    /// If any business rules were violated an `AggregateError` should be returned to warn the user.
    ///
    /// # Examples
    /// ```
    /// # use cqrs_es::{Command,AggregateError};
    /// # use cqrs_es::doc::{Customer, CustomerEvent, NameAdded};
    /// # pub struct AddCustomerName {
    /// #     pub changed_name: String
    /// # }
    /// impl Command<Customer, CustomerEvent> for AddCustomerName {
    ///     fn handle(self, customer: &Customer) -> Result<Vec<CustomerEvent>, AggregateError> {
    ///         if customer.name.as_str() != "" {
    ///             return Err(AggregateError::new("a name has already been added for this customer"));
    ///         }
    ///         let payload = NameAdded {
    ///             changed_name: self.changed_name
    ///         };
    ///         Ok(vec![CustomerEvent::NameAdded(payload)])
    ///     }
    /// }
    /// ```
    fn handle(self, aggregate: &A) -> Result<Vec<E>, AggregateError>;
}