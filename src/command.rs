use crate::aggregate::{Aggregate, AggregateError};
use crate::event::DomainEvent;

/// A `Command` represents a request to modify the state of an `Aggregate` by producing
/// `DomainEvent`s.
///
/// A `Command` is always named in the imperative, e.g.,
/// - `GrantAdminPrivileges`
/// - `ChangeEmailAddress`
/// - `AddDependency`
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
    fn handle(&self, aggregate: &mut A) -> Result<Vec<E>, AggregateError>;
}