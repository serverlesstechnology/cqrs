use serde::{
    Deserialize,
    Serialize,
};
use std::fmt::Debug;

use crate::{
    EventContext,
    IEventConsumer,
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
