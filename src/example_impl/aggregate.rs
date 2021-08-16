use serde::{
    Deserialize,
    Serialize,
};
use std::fmt::Debug;

use crate::{
    AggregateError,
    IAggregate,
};

use super::{
    commands::*,
    events::*,
};

#[derive(
    Debug,
    PartialEq,
    Default,
    Clone,
    Serialize,
    Deserialize
)]
pub struct Customer {
    pub customer_id: String,
    pub name: String,
    pub email: String,
    pub addresses: Vec<String>,
}

impl IAggregate for Customer {
    type Command = CustomerCommand;

    type Event = CustomerEvent;

    fn aggregate_type() -> &'static str {
        "customer"
    }

    fn handle(
        &self,
        command: Self::Command,
    ) -> Result<Vec<Self::Event>, AggregateError> {
        match command {
            CustomerCommand::AddCustomerName(payload) => {
                if self.name.as_str() != "" {
                    return Err(AggregateError::new(
                        "a name has already been added for this \
                         customer",
                    ));
                }

                let payload = NameAdded {
                    changed_name: payload.changed_name,
                };

                Ok(vec![CustomerEvent::NameAdded(payload)])
            },
            CustomerCommand::UpdateEmail(payload) => {
                let payload = EmailUpdated {
                    new_email: payload.new_email,
                };

                Ok(vec![CustomerEvent::EmailUpdated(
                    payload,
                )])
            },
            CustomerCommand::AddAddress(payload) => {
                if self
                    .addresses
                    .iter()
                    .any(|i| payload.new_address.eq(i))
                {
                    return Err(AggregateError::new(
                        "this address has already been added for \
                         this customer",
                    ));
                }

                let payload = AddressUpdated {
                    new_address: payload.new_address,
                };

                Ok(vec![CustomerEvent::AddressUpdated(
                    payload,
                )])
            },
        }
    }

    fn apply(
        &mut self,
        event: &Self::Event,
    ) {
        match event {
            CustomerEvent::NameAdded(payload) => {
                self.name = payload.changed_name.clone();
            },
            CustomerEvent::EmailUpdated(payload) => {
                self.email = payload.new_email.clone();
            },
            CustomerEvent::AddressUpdated(payload) => {
                self.addresses
                    .push(payload.new_address.clone())
            },
        }
    }
}
