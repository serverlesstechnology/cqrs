use crate::{
    errors::AggregateError,
    events::IEvent,
};

use super::i_command::ICommand;

/// ICommandHandler
pub trait ICommandHandler<C: ICommand, E: IEvent> {
    /// handle inbound command and return a vector of events or an
    /// error
    fn handle(
        &self,
        command: C,
    ) -> Result<Vec<E>, AggregateError>;
}
