use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{
        Arc,
        RwLock,
    },
};

use serde::{
    Deserialize,
    Serialize,
};
use static_assertions::assert_impl_all;

use cqrs_es2::{
    memory_store::EventStore as MemoryEventStore,
    test::TestFramework,
    AggregateError,
    CqrsFramework,
    EventEnvelope,
    IAggregate,
    IDomainCommand,
    IDomainEvent,
    IEventStore,
    IQueryProcessor,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestAggregate {
    id: String,
    description: String,
    tests: Vec<String>,
}

impl IAggregate for TestAggregate {
    type Command = TestCommand;
    type Event = TestEvent;

    fn aggregate_type() -> &'static str {
        "TestAggregate"
    }

    fn handle(
        &self,
        command: TestCommand,
    ) -> Result<Vec<TestEvent>, AggregateError> {
        match &command {
            TestCommand::CreateTest(command) => {
                let event = TestEvent::Created(Created {
                    id: command.id.to_string(),
                });
                Ok(vec![event])
            },

            TestCommand::ConfirmTest(command) => {
                for test in &self.tests {
                    if test == &command.test_name {
                        return Err(AggregateError::new(
                            "test already performed",
                        ));
                    }
                }
                let event = TestEvent::Tested(Tested {
                    test_name: command.test_name.to_string(),
                });
                Ok(vec![event])
            },

            TestCommand::DoSomethingElse(command) => {
                let event = TestEvent::SomethingElse(SomethingElse {
                    description: command.description.clone(),
                });
                Ok(vec![event])
            },
        }
    }

    fn apply(
        &mut self,
        event: &Self::Event,
    ) {
        match event {
            TestEvent::Created(e) => {
                self.id = e.id.clone();
            },
            TestEvent::Tested(e) => {
                self.tests.push(e.test_name.clone());
            },
            TestEvent::SomethingElse(e) => {
                self.description = e.description.clone();
            },
        }
    }
}

impl Default for TestAggregate {
    fn default() -> Self {
        TestAggregate {
            id: "".to_string(),
            description: "".to_string(),
            tests: Vec::new(),
        }
    }
}

impl Clone for TestAggregate {
    fn clone(&self) -> Self {
        TestAggregate {
            id: self.id.clone(),
            description: self.description.clone(),
            tests: self.tests.clone(),
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
pub enum TestEvent {
    Created(Created),
    Tested(Tested),
    SomethingElse(SomethingElse),
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct Created {
    pub id: String,
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct Tested {
    pub test_name: String,
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct SomethingElse {
    pub description: String,
}

impl IDomainEvent for TestEvent {}

#[derive(Debug, PartialEq)]
pub enum TestCommand {
    CreateTest(CreateTest),
    ConfirmTest(ConfirmTest),
    DoSomethingElse(DoSomethingElse),
}

#[derive(Debug, PartialEq)]
pub struct CreateTest {
    pub id: String,
}

#[derive(Debug, PartialEq)]
pub struct ConfirmTest {
    pub test_name: String,
}

#[derive(Debug, PartialEq)]
pub struct DoSomethingElse {
    pub description: String,
}

impl IDomainCommand for TestCommand {}

struct TestView {
    events: Arc<RwLock<Vec<EventEnvelope<TestAggregate>>>>,
}

impl TestView {
    fn new(
        events: Arc<RwLock<Vec<EventEnvelope<TestAggregate>>>>
    ) -> Self {
        TestView { events }
    }
}

impl IQueryProcessor<TestAggregate> for TestView {
    fn dispatch(
        &mut self,
        _aggregate_id: &str,
        events: &[EventEnvelope<TestAggregate>],
    ) {
        for event in events {
            let mut event_list = self.events.write().unwrap();
            event_list.push(event.clone());
        }
    }
}

pub type TestEventEnvelope = EventEnvelope<TestAggregate>;

assert_impl_all!(TestAggregate: IAggregate);

assert_impl_all!(
    MemoryEventStore::<TestAggregate>: IEventStore::<TestAggregate>
);

fn metadata() -> HashMap<String, String> {
    let now = "2021-03-18T12:32:45.930Z".to_string();
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), now);
    metadata
}

#[test]
fn test_mem_store() {
    let mut event_store =
        MemoryEventStore::<TestAggregate>::default();
    let id = "test_id_A";
    let initial_events = event_store.load_events(&id);
    assert_eq!(0, initial_events.len());
    let agg_context = event_store.load_aggregate(&id);

    event_store
        .commit(
            vec![TestEvent::Created(Created {
                id: "test_event_A".to_string(),
            })],
            agg_context,
            metadata(),
        )
        .unwrap();
    let stored_events = event_store.load_events(&id);
    assert_eq!(1, stored_events.len());
    let agg_context = event_store.load_aggregate(&id);

    event_store
        .commit(
            vec![
                TestEvent::Tested(Tested {
                    test_name: "test A".to_string(),
                }),
                TestEvent::Tested(Tested {
                    test_name: "test B".to_string(),
                }),
                TestEvent::SomethingElse(SomethingElse {
                    description: "something else happening here"
                        .to_string(),
                }),
            ],
            agg_context,
            metadata(),
        )
        .unwrap();
    let stored_envelopes = event_store.load_events(&id);

    let mut agg = TestAggregate::default();
    for stored_envelope in stored_envelopes {
        let event = stored_envelope.payload;
        agg.apply(&event);
    }
    println!("{:#?}", agg);
}

type ThisTestFramework = TestFramework<TestAggregate>;

#[test]
fn test_framework_test() {
    let test_name = "test A";
    let test_framework = ThisTestFramework::default();

    test_framework
        .given(vec![TestEvent::Created(Created {
            id: "test_id_A".to_string(),
        })])
        .when(TestCommand::ConfirmTest(ConfirmTest {
            test_name: test_name.to_string(),
        }))
        .then_expect_events(vec![TestEvent::Tested(Tested {
            test_name: test_name.to_string(),
        })]);

    test_framework
        .given(vec![TestEvent::Tested(Tested {
            test_name: test_name.to_string(),
        })])
        .when(TestCommand::ConfirmTest(ConfirmTest {
            test_name: test_name.to_string(),
        }))
        .then_expect_error("test already performed")
}

#[test]
#[should_panic]
fn test_framework_failure_test() {
    let test_name = "test A";
    let test_framework = ThisTestFramework::default();

    test_framework
        .given(vec![TestEvent::Tested(Tested {
            test_name: test_name.to_string(),
        })])
        .when(TestCommand::ConfirmTest(ConfirmTest {
            test_name: test_name.to_string(),
        }))
        .then_expect_events(vec![TestEvent::Tested(Tested {
            test_name: test_name.to_string(),
        })]);
}

#[test]
#[should_panic]
fn test_framework_failure_test_b() {
    let test_name = "test A";
    let test_framework = ThisTestFramework::default();

    test_framework
        .given(vec![TestEvent::Created(Created {
            id: "test_id_A".to_string(),
        })])
        .when(TestCommand::ConfirmTest(ConfirmTest {
            test_name: test_name.to_string(),
        }))
        .then_expect_error("some error message")
}

#[test]
fn framework_test() {
    let event_store = MemoryEventStore::default();
    let stored_events = event_store.get_events();

    let delivered_events = Default::default();
    let view = TestView::new(Arc::clone(&delivered_events));

    let mut cqrs =
        CqrsFramework::new(event_store, vec![Box::new(view)]);
    let uuid = uuid::Uuid::new_v4().to_string();
    let id = uuid.clone();
    let metadata = metadata();
    cqrs.execute_with_metadata(
        &id,
        TestCommand::ConfirmTest(ConfirmTest {
            test_name: uuid.clone(),
        }),
        metadata,
    )
    .unwrap_or_default();

    assert_eq!(1, stored_events.read().unwrap().len());
    assert_eq!(
        1,
        delivered_events.read().unwrap().len()
    );

    let test = "TEST_A";
    let id = uuid.clone();
    cqrs.execute(
        &id,
        TestCommand::ConfirmTest(ConfirmTest {
            test_name: test.to_string(),
        }),
    )
    .unwrap_or_default();

    assert_eq!(
        2,
        delivered_events.read().unwrap().len()
    );
    let stored_event_count = stored_events
        .read()
        .unwrap()
        .get(uuid.clone().as_str())
        .unwrap()
        .len();
    assert_eq!(2, stored_event_count);

    let id = uuid.clone();
    let err = cqrs
        .execute(
            &id,
            TestCommand::ConfirmTest(ConfirmTest {
                test_name: test.to_string(),
            }),
        )
        .unwrap_err();
    assert_eq!(
        AggregateError::new("test already performed"),
        err
    );

    assert_eq!(
        2,
        delivered_events.read().unwrap().len()
    );
    let stored_event_count = stored_events
        .read()
        .unwrap()
        .get(uuid.clone().as_str())
        .unwrap()
        .len();
    assert_eq!(2, stored_event_count);
}
