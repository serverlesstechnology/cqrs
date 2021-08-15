use serde::{
    Deserialize,
    Serialize,
};
use std::fmt::Debug;

use crate::{
    EventContext,
    IQuery,
};

use super::{
    aggregate::Customer,
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
}

impl IQuery<Customer> for CustomerContactQuery {
    fn query_type() -> &'static str {
        "customer_contact_query"
    }

    fn update(
        &mut self,
        event: &EventContext<Customer>,
    ) {
        match &event.payload {
            CustomerEvent::NameAdded(payload) => {
                self.name = payload.changed_name.clone();
            },
            CustomerEvent::EmailUpdated(payload) => {
                self.email = payload.new_email.clone();
            },
        }
    }
}
