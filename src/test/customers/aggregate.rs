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
            CustomerCommand::UpdateEmail(_) => Ok(Default::default()),
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
        }
    }
}
