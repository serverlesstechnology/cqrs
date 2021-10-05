use serde::{Deserialize, Serialize};

use crate::{Aggregate, AggregateError, DomainEvent};




#[derive(Debug,Default,Serialize,Deserialize,PartialEq,Clone)]
pub struct MyEvents;
impl DomainEvent for MyEvents {
    fn event_type(&self) -> &'static str {
        todo!()
    }
    fn event_version(&self) -> &'static str {
        todo!()
    }
}
#[derive(Debug,Default,Serialize,Deserialize)]
pub struct MyCommands;
#[derive(Debug,Default,Serialize,Deserialize)]
pub struct MyAggregate;
impl Aggregate for MyAggregate {
    type Command = MyCommands;
    type Event = MyEvents;

    fn aggregate_type() -> &'static str {
        todo!()
    }

    fn handle(&self, _command: Self::Command) -> Result<Vec<Self::Event>, AggregateError> {
        todo!()
    }

    fn apply(&mut self, _event: Self::Event) {
        todo!()
    }
}



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

    fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, AggregateError> {
        match command {
            CustomerCommand::AddCustomerName(payload) => {
                if self.name.as_str() != "" {
                    return Err(AggregateError::new(
                        "a name has already been added for this customer",
                    ));
                }
                let payload = NameAdded {
                    changed_name: payload.changed_name,
                };
                Ok(vec![CustomerEvent::NameAdded(payload)])
            }
            CustomerCommand::UpdateEmail(_) => Ok(Default::default()),
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            CustomerEvent::NameAdded(payload) => {
                self.name = payload.changed_name;
            }
            CustomerEvent::EmailUpdated(payload) => {
                self.email = payload.new_email;
            }
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CustomerEvent {
    NameAdded(NameAdded),
    EmailUpdated(EmailUpdated),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NameAdded {
    pub changed_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EmailUpdated {
    pub new_email: String,
}

impl DomainEvent for CustomerEvent {
    fn event_type(&self) -> &'static str {
        match self {
            CustomerEvent::NameAdded(_) => "NameAdded",
            CustomerEvent::EmailUpdated(_) => "EmailUpdated",
        }
    }

    fn event_version(&self) -> &'static str {
        "1.0"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CustomerCommand {
    AddCustomerName(AddCustomerName),
    UpdateEmail(UpdateEmail),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AddCustomerName {
    pub changed_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UpdateEmail {
    pub new_email: String,
}

#[cfg(test)]
mod doc_tests {
    use crate::test::TestFramework;

    use super::*;

    type CustomerTestFramework = TestFramework<Customer>;

    #[test]
    fn test_add_name() {
        CustomerTestFramework::default()
            .given_no_previous_events()
            .when(CustomerCommand::AddCustomerName(AddCustomerName {
                changed_name: "John Doe".to_string(),
            }))
            .then_expect_events(vec![CustomerEvent::NameAdded(NameAdded {
                changed_name: "John Doe".to_string(),
            })]);
    }

    #[test]
    fn test_add_name_again() {
        CustomerTestFramework::default()
            .given(vec![CustomerEvent::NameAdded(NameAdded {
                changed_name: "John Doe".to_string(),
            })])
            .when(CustomerCommand::AddCustomerName(AddCustomerName {
                changed_name: "John Doe".to_string(),
            }))
            .then_expect_error("a name has already been added for this customer");
    }
}
