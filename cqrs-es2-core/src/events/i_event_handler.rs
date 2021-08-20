use super::i_event::IEvent;

/// Event handlers are usually the aggregates. It applies the
/// events to its state.
///
/// # Example
///
/// For illustration only:
///
/// ```rust
/// use cqrs_es2_core::{
///     example_impl::CustomerEvent,
///     IEventHandler,
/// };
///
/// pub struct CustomerEventHandler {
///     pub name: String,
///     pub email: String,
///     pub addresses: Vec<String>,
/// };
///
/// impl IEventHandler<CustomerEvent> for CustomerEventHandler {
///     fn apply(
///         &mut self,
///         event: &CustomerEvent,
///     ) {
///         match event {
///             CustomerEvent::NameAdded(payload) => {
///                 self.name = payload.changed_name.clone();
///             },
///             CustomerEvent::EmailUpdated(payload) => {
///                 self.email = payload.new_email.clone();
///             },
///             CustomerEvent::AddressUpdated(payload) => {
///                 self.addresses
///                     .push(payload.new_address.clone())
///             },
///         }
///     }
/// }
/// ```
pub trait IEventHandler<E: IEvent> {
    /// handle events
    fn apply(
        &mut self,
        event: &E,
    );
}
