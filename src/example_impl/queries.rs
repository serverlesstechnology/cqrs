use serde::{
    Deserialize,
    Serialize,
};
use std::{
    fmt::Debug,
    sync::{
        Arc,
        RwLock,
    },
};

use crate::{
    AggregateError,
    EventContext,
    IEventConsumer,
    IEventDispatcher,
    IQuery,
};

use super::{
    commands::CustomerCommand,
    events::CustomerEvent,
};

#[derive(
    Debug,
    PartialEq,
    Default,
    Clone,
    Serialize,
    Deserialize
)]
pub struct CustomerContactQuery {
    pub name: String,
    pub email: String,
    pub latest_address: String,
}

impl IQuery<CustomerCommand, CustomerEvent> for CustomerContactQuery {
    fn query_type() -> &'static str {
        "customer_contact_query"
    }
}

impl IEventConsumer<CustomerCommand, CustomerEvent>
    for CustomerContactQuery
{
    fn update(
        &mut self,
        event: &EventContext<CustomerCommand, CustomerEvent>,
    ) {
        match &event.payload {
            CustomerEvent::NameAdded(payload) => {
                self.name = payload.changed_name.clone();
            },
            CustomerEvent::EmailUpdated(payload) => {
                self.email = payload.new_email.clone();
            },
            CustomerEvent::AddressUpdated(payload) => {
                self.latest_address = payload.new_address.clone();
            },
        }
    }
}

pub struct TestView {
    events: Arc<
        RwLock<Vec<EventContext<CustomerCommand, CustomerEvent>>>,
    >,
}

impl TestView {
    pub fn new(
        events: Arc<
            RwLock<Vec<EventContext<CustomerCommand, CustomerEvent>>>,
        >
    ) -> Self {
        TestView { events }
    }
}

impl IEventDispatcher<CustomerCommand, CustomerEvent> for TestView {
    fn dispatch(
        &mut self,
        _aggregate_id: &str,
        events: &[EventContext<CustomerCommand, CustomerEvent>],
    ) -> Result<(), AggregateError> {
        for event in events {
            let mut event_list = self.events.write().unwrap();
            event_list.push(event.clone());
        }

        Ok(())
    }
}
