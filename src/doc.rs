use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{Aggregate, AggregateError, DomainEvent, UserErrorPayload};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum MyEvents {
    SomethingWasDone,
}
impl DomainEvent for MyEvents {
    fn event_type(&self) -> &'static str {
        todo!()
    }
    fn event_version(&self) -> &'static str {
        todo!()
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum MyCommands {
    DoSomething,
    BadCommand,
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MyAggregate;

#[async_trait]
impl Aggregate for MyAggregate {
    type Command = MyCommands;
    type Event = MyEvents;
    type Error = UserErrorPayload;

    fn aggregate_type() -> &'static str {
        todo!()
    }

    async fn handle(
        &self,
        command: Self::Command,
    ) -> Result<Vec<Self::Event>, AggregateError<UserErrorPayload>> {
        match command {
            MyCommands::DoSomething => Ok(vec![MyEvents::SomethingWasDone]),
            MyCommands::BadCommand => Err("the expected error message".into()),
        }
    }

    fn apply(&mut self, _event: Self::Event) {}
}

#[derive(Serialize, Deserialize)]
pub struct Customer {
    pub customer_id: String,
    pub name: String,
    pub email: String,
}

#[async_trait]
impl Aggregate for Customer {
    type Command = CustomerCommand;
    type Event = CustomerEvent;
    type Error = UserErrorPayload;

    fn aggregate_type() -> &'static str {
        "customer"
    }

    async fn handle(
        &self,
        command: Self::Command,
    ) -> Result<Vec<Self::Event>, AggregateError<Self::Error>> {
        match command {
            CustomerCommand::AddCustomerName { changed_name } => {
                if self.name.as_str() != "" {
                    return Err("a name has already been added for this customer".into());
                }
                Ok(vec![CustomerEvent::NameAdded { changed_name }])
            }
            CustomerCommand::UpdateEmail { .. } => Ok(Default::default()),
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            CustomerEvent::NameAdded { changed_name } => {
                self.name = changed_name;
            }
            CustomerEvent::EmailUpdated { new_email } => {
                self.email = new_email;
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
    NameAdded { changed_name: String },
    EmailUpdated { new_email: String },
}

impl DomainEvent for CustomerEvent {
    fn event_type(&self) -> &'static str {
        match self {
            CustomerEvent::NameAdded { .. } => "NameAdded",
            CustomerEvent::EmailUpdated { .. } => "EmailUpdated",
        }
    }

    fn event_version(&self) -> &'static str {
        "1.0"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CustomerCommand {
    AddCustomerName { changed_name: String },
    UpdateEmail { new_email: String },
}

#[cfg(test)]
mod doc_tests {
    use crate::test::TestFramework;

    use super::*;

    type CustomerTestFramework = TestFramework<Customer>;

    #[tokio::test]
    async fn test_add_name() {
        CustomerTestFramework::default()
            .given_no_previous_events()
            .when(CustomerCommand::AddCustomerName {
                changed_name: "John Doe".to_string(),
            })
            .await
            .then_expect_events(vec![CustomerEvent::NameAdded {
                changed_name: "John Doe".to_string(),
            }]);
    }

    #[tokio::test]
    async fn test_add_name_again() {
        CustomerTestFramework::default()
            .given(vec![CustomerEvent::NameAdded {
                changed_name: "John Doe".to_string(),
            }])
            .when(CustomerCommand::AddCustomerName {
                changed_name: "John Doe".to_string(),
            })
            .await
            .then_expect_error("a name has already been added for this customer");
    }
}
