use crate::{
    errors::Error,
    events::IEvent,
};

use super::i_command::ICommand;

/// Command handlers are usually the aggregates. It consumes the
/// command and emit all events resulting from this command.
///
/// # Example
///
/// For illustration only:
///
/// ```rust
/// use cqrs_es2::{
///     example_impl::{
///         AddressUpdated,
///         CustomerCommand,
///         CustomerEvent,
///         EmailUpdated,
///         NameAdded,
///     },
///     Error,
///     ICommandHandler,
/// };
///
/// pub struct CustomerCommandHandler {};
///
/// impl ICommandHandler<CustomerCommand, CustomerEvent>
///     for CustomerCommandHandler
/// {
///     fn handle(
///         &self,
///         command: CustomerCommand,
///     ) -> Result<Vec<CustomerEvent>, Error> {
///         match command {
///             CustomerCommand::AddCustomerName(payload) => {
///                 let payload = NameAdded {
///                     changed_name: payload.changed_name,
///                 };
///
///                 Ok(vec![CustomerEvent::NameAdded(payload)])
///             },
///             CustomerCommand::UpdateEmail(payload) => {
///                 let payload = EmailUpdated {
///                     new_email: payload.new_email,
///                 };
///
///                 Ok(vec![CustomerEvent::EmailUpdated(
///                     payload,
///                 )])
///             },
///             CustomerCommand::AddAddress(payload) => {
///                 let payload = AddressUpdated {
///                     new_address: payload.new_address,
///                 };
///
///                 Ok(vec![CustomerEvent::AddressUpdated(
///                     payload,
///                 )])
///             },
///         }
///     }
/// }
/// ```
pub trait ICommandHandler<C: ICommand, E: IEvent> {
    /// handle inbound command and return a vector of events or an
    /// error
    fn handle(
        &self,
        command: C,
    ) -> Result<Vec<E>, Error>;
}
