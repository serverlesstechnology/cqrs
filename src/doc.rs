use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};

use crate::persist::{
    PersistedEventRepository, PersistenceError, ReplayStream, SerializedEvent, SerializedSnapshot,
};
use crate::{Aggregate, DomainEvent, EventEnvelope, Query};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum MyEvents {
    SomethingWasDone,
}
impl DomainEvent for MyEvents {
    fn event_type(&self) -> String {
        match self {
            MyEvents::SomethingWasDone => "SomethingWasDone".to_string(),
        }
    }
    fn event_version(&self) -> String {
        "0.1.0".to_string()
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
    type Error = MyUserError;
    type Services = MyService;

    fn aggregate_type() -> String {
        "MyAggregate".to_string()
    }

    async fn handle(
        &self,
        command: Self::Command,
        _service: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error> {
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

#[derive(Debug, PartialEq)]
pub struct MyUserError(pub String);

impl Display for MyUserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MyUserError {}

impl From<&str> for MyUserError {
    fn from(msg: &str) -> Self {
        Self(msg.to_string())
    }
}

#[derive(Debug, Clone, Default)]
pub struct MyService;

#[derive(Debug, Default)]
pub struct MyQuery;

#[async_trait]
impl Query<MyAggregate> for MyQuery {
    async fn dispatch(&self, _aggregate_id: &str, _events: &[EventEnvelope<MyAggregate>]) {}
}

#[async_trait]
impl Aggregate for Customer {
    type Command = CustomerCommand;
    type Event = CustomerEvent;
    type Error = CustomerError;
    type Services = CustomerService;

    fn aggregate_type() -> String {
        "Customer".to_string()
    }

    async fn handle(
        &self,
        command: Self::Command,
        _service: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            CustomerCommand::AddCustomerName { name: changed_name } => {
                if self.name.as_str() != "" {
                    return Err("a name has already been added for this customer".into());
                }
                Ok(vec![CustomerEvent::NameAdded { name: changed_name }])
            }
            CustomerCommand::UpdateEmail { new_email } => {
                Ok(vec![CustomerEvent::EmailUpdated { new_email }])
            }
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            CustomerEvent::NameAdded { name: changed_name } => {
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
pub struct CustomerError(String);

impl From<&str> for CustomerError {
    fn from(message: &str) -> Self {
        CustomerError(message.to_string())
    }
}

impl Display for CustomerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CustomerError {}

#[derive(Clone, Default)]
pub struct CustomerService;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CustomerEvent {
    NameAdded { name: String },
    EmailUpdated { new_email: String },
}

impl DomainEvent for CustomerEvent {
    fn event_type(&self) -> String {
        match self {
            CustomerEvent::NameAdded { .. } => "NameAdded".to_string(),
            CustomerEvent::EmailUpdated { .. } => "EmailUpdated".to_string(),
        }
    }

    fn event_version(&self) -> String {
        "1.0".to_string()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CustomerCommand {
    AddCustomerName { name: String },
    UpdateEmail { new_email: String },
}

#[cfg(test)]
mod doc_tests {
    use crate::test::TestFramework;

    use super::*;

    type CustomerTestFramework = TestFramework<Customer>;

    #[test]
    fn test_add_name() {
        CustomerTestFramework::with(CustomerService::default())
            .given_no_previous_events()
            .when(CustomerCommand::AddCustomerName {
                name: "John Doe".to_string(),
            })
            .then_expect_events(vec![CustomerEvent::NameAdded {
                name: "John Doe".to_string(),
            }]);
    }

    #[test]
    fn test_add_name_again() {
        CustomerTestFramework::with(CustomerService::default())
            .given(vec![CustomerEvent::NameAdded {
                name: "John Doe".to_string(),
            }])
            .when(CustomerCommand::AddCustomerName {
                name: "John Doe".to_string(),
            })
            .then_expect_error_message("a name has already been added for this customer");
    }
}

pub struct MyRepository;
#[async_trait]
impl PersistedEventRepository for MyRepository {
    async fn get_events<A: Aggregate>(
        &self,
        _aggregate_id: &str,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        todo!()
    }

    async fn get_last_events<A: Aggregate>(
        &self,
        _aggregate_id: &str,
        _number_events: usize,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        todo!()
    }

    async fn get_snapshot<A: Aggregate>(
        &self,
        _aggregate_id: &str,
    ) -> Result<Option<SerializedSnapshot>, PersistenceError> {
        todo!()
    }

    async fn persist<A: Aggregate>(
        &self,
        _events: &[SerializedEvent],
        _snapshot_update: Option<(String, Value, usize)>,
    ) -> Result<(), PersistenceError> {
        todo!()
    }

    async fn stream_events<A: Aggregate>(
        &self,
        _aggregate_id: &str,
    ) -> Result<ReplayStream, PersistenceError> {
        todo!()
    }

    async fn stream_all_events<A: Aggregate>(&self) -> Result<ReplayStream, PersistenceError> {
        todo!()
    }
}
