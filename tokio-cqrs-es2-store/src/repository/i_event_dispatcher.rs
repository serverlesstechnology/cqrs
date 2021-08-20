use async_trait::async_trait;

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
/// use async_trait::async_trait;
/// use cqrs_es2_core::{
///     example_impl::{
///         CustomerCommand,
///         CustomerEvent,
///     },
///     Error,
///     EventContext,
/// };
///
/// use tokio_cqrs_es2_store::IEventDispatcher;
///
/// pub struct CustomerEventDispatcher {
///     pub name: String,
///     pub email: String,
///     pub latest_address: String,
/// };
///
/// #[async_trait]
/// impl IEventDispatcher<CustomerCommand, CustomerEvent>
///     for CustomerEventDispatcher
/// {
///     async fn dispatch(
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
#[async_trait]
pub trait IEventDispatcher<C: ICommand, E: IEvent>: Send {
    /// Events will be dispatched here immediately after being
    /// committed for the downstream queries to be updated.
    async fn dispatch(
        &mut self,
        aggregate_id: &str,
        events: &[EventContext<C, E>],
    ) -> Result<(), Error>;
}
