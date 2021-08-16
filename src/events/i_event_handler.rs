use super::i_event::IEvent;

/// IEventHandler
pub trait IEventHandler<E: IEvent> {
    /// Consume events
    fn apply(
        &mut self,
        event: &E,
    );
}
