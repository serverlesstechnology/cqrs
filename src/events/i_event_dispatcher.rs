use crate::{
    commands::ICommand,
    errors::AggregateError,
};

use super::{
    event_context::EventContext,
    i_event::IEvent,
};

/// IEventDispatcher
pub trait IEventDispatcher<C: ICommand, E: IEvent> {
    /// Events will be dispatched here immediately after being
    /// committed for the downstream queries to be updated.
    fn dispatch(
        &mut self,
        aggregate_id: &str,
        events: &[EventContext<C, E>],
    ) -> Result<(), AggregateError>;
}
