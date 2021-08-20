use log::info;

use cqrs_es2::{
    Error,
    EventContext,
    IEventDispatcher,
};

use super::super::{
    commands::BankAccountCommand,
    events::BankAccountEvent,
};

pub struct LoggingDispatcher {}

impl LoggingDispatcher {
    pub fn new() -> Self {
        Self {}
    }
}

impl IEventDispatcher<BankAccountCommand, BankAccountEvent>
    for LoggingDispatcher
{
    fn dispatch(
        &mut self,
        aggregate_id: &str,
        events: &[EventContext<
            BankAccountCommand,
            BankAccountEvent,
        >],
    ) -> Result<(), Error> {
        for event in events {
            let payload =
                serde_json::to_string_pretty(&event.payload).unwrap();
            info!(
                "dispatching event '{}' for aggregate '{}' with \
                 sequence '{}'",
                payload, aggregate_id, event.sequence
            );
        }

        Ok(())
    }
}
