use serde::{
    Deserialize,
    Serialize,
};
use std::fmt::Debug;

use crate::{
    Error,
    IAggregate,
    ICommandHandler,
    IEventHandler,
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

impl IAggregate<CustomerCommand, CustomerEvent> for Customer {
    fn aggregate_type() -> &'static str {
        "customer"
    }
}

impl ICommandHandler<CustomerCommand, CustomerEvent> for Customer {
    fn handle(
        &self,
        command: CustomerCommand,
    ) -> Result<Vec<CustomerEvent>, Error> {
        match command {
            CustomerCommand::AddCustomerName(payload) => {
                if self.name.as_str() != "" {
                    return Err(Error::new(
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
                    return Err(Error::new(
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
}

impl IEventHandler<CustomerEvent> for Customer {
    fn apply(
        &mut self,
        event: &CustomerEvent,
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
