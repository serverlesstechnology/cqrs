use crate::commands::ICommand;

use super::{
    event_context::EventContext,
    i_event::IEvent,
};

/// IEventContextHandler
pub trait IEventConsumer<C: ICommand, E: IEvent> {
    /// Each implemented query is responsible for updating its stated
    /// based on events passed via this method.
    fn update(
        &mut self,
        event: &EventContext<C, E>,
    );
}
