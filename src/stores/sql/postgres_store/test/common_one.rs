use std::collections::HashMap;

pub static CONNECTION_STRING: &str =
    "postgresql://test_user:test_pass@localhost:5432/test";

// #[derive(Debug, Serialize, Deserialize)]
// pub struct TestAggregate {
//     id: String,
//     description: String,
//     tests: Vec<String>,
// }

// impl IAggregate for TestAggregate {
//     type Command = TestCommand;
//     type Event = TestEvent;

//     fn aggregate_type() -> &'static str {
//         "TestAggregate"
//     }

//     fn handle(
//         &self,
//         command: Self::Command,
//     ) -> Result<Vec<Self::Event>, Error> {
//         match command {
//             TestCommand::CreateTest(command) => {
//                 let event = TestEvent::Created(Created {
//                     id: command.id.to_string(),
//                 });
//                 Ok(vec![event])
//             },
//             TestCommand::ConfirmTest(command) => {
//                 for test in &self.tests {
//                     if test == &command.test_name {
//                         return Err(Error::new(
//                             "test already performed",
//                         ));
//                     }
//                 }
//                 let event = TestEvent::Tested(Tested {
//                     test_name: command.test_name,
//                 });
//                 Ok(vec![event])
//             },
//             TestCommand::DoSomethingElse(command) => {
//                 let event = TestEvent::SomethingElse(SomethingElse
// {                     description: command.description.clone(),
//                 });
//                 Ok(vec![event])
//             },
//         }
//     }

//     fn apply(
//         &mut self,
//         e: &Self::Event,
//     ) {
//         match e {
//             TestEvent::Created(e) => {
//                 self.id = e.id.clone();
//             },
//             TestEvent::Tested(e) => {
//                 self.tests.push(e.test_name.clone())
//             },
//             TestEvent::SomethingElse(e) => {
//                 self.description = e.description.clone();
//             },
//         }
//     }
// }

// impl Default for TestAggregate {
//     fn default() -> Self {
//         TestAggregate {
//             id: "".to_string(),
//             description: "".to_string(),
//             tests: Vec::new(),
//         }
//     }
// }

// impl Clone for TestAggregate {
//     fn clone(&self) -> Self {
//         TestAggregate {
//             id: self.id.clone(),
//             description: self.description.clone(),
//             tests: self.tests.clone(),
//         }
//     }
// }

// #[derive(
//     Debug,
//     Serialize,
//     Deserialize,
//     Clone,
//     PartialEq
// )]
// pub enum TestEvent {
//     Created(Created),
//     Tested(Tested),
//     SomethingElse(SomethingElse),
// }

// #[derive(
//     Debug,
//     Serialize,
//     Deserialize,
//     Clone,
//     PartialEq
// )]
// pub struct Created {
//     pub id: String,
// }

// #[derive(
//     Debug,
//     Serialize,
//     Deserialize,
//     Clone,
//     PartialEq
// )]
// pub struct Tested {
//     pub test_name: String,
// }

// #[derive(
//     Debug,
//     Serialize,
//     Deserialize,
//     Clone,
//     PartialEq
// )]
// pub struct SomethingElse {
//     pub description: String,
// }

// impl IEvent for TestEvent {}

// #[derive(Debug, PartialEq)]
// pub enum TestCommand {
//     CreateTest(CreateTest),
//     ConfirmTest(ConfirmTest),
//     DoSomethingElse(DoSomethingElse),
// }

// #[derive(Debug, PartialEq)]
// pub struct CreateTest {
//     pub id: String,
// }

// #[derive(Debug, PartialEq)]
// pub struct ConfirmTest {
//     pub test_name: String,
// }

// #[derive(Debug, PartialEq)]
// pub struct DoSomethingElse {
//     pub description: String,
// }

// impl ICommand for TestCommand {}

// pub struct TestQuery {
//     events: Rc<RwLock<Vec<EventContext<TestAggregate>>>>,
// }

// impl TestQuery {
//     pub fn new(
//         events: Rc<RwLock<Vec<EventContext<TestAggregate>>>>
//     ) -> Self {
//         TestQuery { events }
//     }
// }

// impl IQueryHandler<TestAggregate> for TestQuery {
//     fn dispatch(
//         &mut self,
//         _aggregate_id: &str,
//         events: &[EventContext<TestAggregate>],
//     ) {
//         for event in events {
//             let mut event_list = self.events.write().unwrap();
//             event_list.push(event.clone());
//         }
//     }
// }

pub fn metadata() -> HashMap<String, String> {
    let now = "2021-03-18T12:32:45.930Z".to_string();
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), now);
    metadata
}
