use crate::commands::ICommand;

use super::{
    event_context::EventContext,
    i_event::IEvent,
};

/// Event consumers are usually the queries. It updates its state with
/// the emitted events.
///
/// # Example
///
/// For illustration only:
///
/// ```rust
/// use cqrs_es2::{
///     example_impl::{
///         CustomerCommand,
///         CustomerEvent,
///     },
///     EventContext,
///     IEventConsumer,
/// };
///
/// pub struct CustomerEventConsumer {
///     pub name: String,
///     pub email: String,
///     pub latest_address: String,
/// };
///
/// impl IEventConsumer<CustomerCommand, CustomerEvent>
///     for CustomerEventConsumer
/// {
///     fn update(
///         &mut self,
///         event: &EventContext<CustomerCommand, CustomerEvent>,
///     ) {
///         match &event.payload {
///             CustomerEvent::NameAdded(payload) => {
///                 self.name = payload.changed_name.clone();
///             },
///             CustomerEvent::EmailUpdated(payload) => {
///                 self.email = payload.new_email.clone();
///             },
///             CustomerEvent::AddressUpdated(payload) => {
///                 self.latest_address = payload.new_address.clone();
///             },
///         }
///     }
/// }
/// ```
pub trait IEventConsumer<C: ICommand, E: IEvent> {
    /// Each implemented query is responsible for updating its stated
    /// based on events passed via this method.
    fn update(
        &mut self,
        event: &EventContext<C, E>,
    );
}
