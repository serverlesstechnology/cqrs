use cqrs_es2_core::{
    Error,
    EventContext,
    ICommand,
    IEvent,
};

/// Event dispatcher are usually the query stores. It updates its
/// query with the emitted events.
///
/// # Example
///
/// For illustration only:
///
/// ```rust
/// use cqrs_es2_core::{
///     example_impl::{
///         CustomerCommand,
///         CustomerEvent,
///     },
///     Error,
///     EventContext,
/// };
///
/// use cqrs_es2_store::IEventDispatcher;
///
/// pub struct CustomerEventDispatcher {
///     pub name: String,
///     pub email: String,
///     pub latest_address: String,
/// };
///
/// impl IEventDispatcher<CustomerCommand, CustomerEvent>
///     for CustomerEventDispatcher
/// {
///     fn dispatch(
///         &mut self,
///         aggregate_id: &str,
///         events: &[EventContext<CustomerCommand, CustomerEvent>],
///     ) -> Result<(), Error> {
///         for event in events {
///             //..
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait IEventDispatcher<C: ICommand, E: IEvent> {
    /// Events will be dispatched here immediately after being
    /// committed for the downstream queries to be updated.
    fn dispatch(
        &mut self,
        aggregate_id: &str,
        events: &[EventContext<C, E>],
    ) -> Result<(), Error>;
}
