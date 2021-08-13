use serde::{
    Deserialize,
    Serialize,
};
use std::fmt::Debug;

use crate::{
    Aggregate,
    AggregateError,
    DomainCommand,
    DomainEvent,
};

#[derive(Serialize, Deserialize)]
pub struct Customer {
    pub customer_id: String,
    pub name: String,
    pub email: String,
}

impl Aggregate for Customer {
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

impl Default for Customer {
    fn default() -> Self {
        Customer {
            customer_id: "".to_string(),
            name: "".to_string(),
            email: "".to_string(),
        }
    }
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub enum CustomerEvent {
    NameAdded(NameAdded),
    EmailUpdated(EmailUpdated),
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct NameAdded {
    pub changed_name: String,
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct EmailUpdated {
    pub new_email: String,
}

impl DomainEvent for CustomerEvent {}

#[derive(Debug, PartialEq)]
pub enum CustomerCommand {
    AddCustomerName(AddCustomerName),
    UpdateEmail(UpdateEmail),
}

#[derive(Debug, PartialEq)]
pub struct AddCustomerName {
    pub changed_name: String,
}

#[derive(Debug, PartialEq)]
pub struct UpdateEmail {
    pub new_email: String,
}

impl DomainCommand for CustomerCommand {}

#[cfg(test)]
mod doc_tests {
    use crate::test::TestFramework;

    use super::*;

    type CustomerTestFramework = TestFramework<Customer>;

    #[test]
    fn test_add_name() {
        CustomerTestFramework::default()
            .given_no_previous_events()
            .when(CustomerCommand::AddCustomerName(
                AddCustomerName {
                    changed_name: "John Doe".to_string(),
                },
            ))
            .then_expect_events(vec![CustomerEvent::NameAdded(
                NameAdded {
                    changed_name: "John Doe".to_string(),
                },
            )]);
    }

    #[test]
    fn test_add_name_again() {
        CustomerTestFramework::default()
            .given(vec![CustomerEvent::NameAdded(
                NameAdded {
                    changed_name: "John Doe".to_string(),
                },
            )])
            .when(CustomerCommand::AddCustomerName(
                AddCustomerName {
                    changed_name: "John Doe".to_string(),
                },
            ))
            .then_expect_error(
                "a name has already been added for this customer",
            );
    }
}
