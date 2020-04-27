use std::collections::HashMap;
use std::rc::Rc;
use std::sync::RwLock;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use static_assertions::assert_impl_all;

use cqrs_es::{Aggregate,
              AggregateError,
              Command,
              CqrsFramework,
              DomainEvent,
              EventStore,
              MessageEnvelope,
              TimeMetadataSupplier,
};
use cqrs_es::mem_store::MemStore;
use cqrs_es::QueryProcessor;
use cqrs_es::test::TestFramework;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestAggregate {
    id: String,
    description: String,
    tests: Vec<String>,
}

impl Aggregate for TestAggregate { fn aggregate_type() -> &'static str { "TestAggregate" } }

impl Default for TestAggregate {
    fn default() -> Self {
        TestAggregate {
            id: "".to_string(),
            description: "".to_string(),
            tests: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TestEvent {
    Created(Created),
    Tested(Tested),
    SomethingElse(SomethingElse),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Created {
    pub id: String
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Tested {
    pub test_name: String
}

impl DomainEvent<TestAggregate> for Tested {
    fn apply(self, aggregate: &mut TestAggregate) {
        aggregate.tests.push(self.test_name);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SomethingElse {
    pub description: String
}

impl DomainEvent<TestAggregate> for TestEvent {
    fn apply(self, aggregate: &mut TestAggregate) {
        match self {
            TestEvent::Created(e) => {
                aggregate.id = e.id;
            }
            TestEvent::Tested(e) => { e.apply(aggregate) }
            TestEvent::SomethingElse(e) => {
                aggregate.description = e.description;
            }
        }
    }
}

pub struct CreateTest {
    pub id: String,
}

impl Command<TestAggregate, TestEvent> for CreateTest {
    fn handle(self, _aggregate: &TestAggregate) -> Result<Vec<TestEvent>, AggregateError> {
        let event = TestEvent::Created(Created { id: self.id.to_string() });
        Ok(vec![event])
    }
}

pub struct ConfirmTest<'a> {
    pub test_name: &'a str,
}

impl<'a> Command<TestAggregate, TestEvent> for ConfirmTest<'a> {
    fn handle(self, aggregate: &TestAggregate) -> Result<Vec<TestEvent>, AggregateError> {
        for test in &aggregate.tests {
            if test == &self.test_name {
                return Err(AggregateError::new("test already performed"));
            }
        }
        let event = TestEvent::Tested(Tested { test_name: self.test_name.to_string() });
        Ok(vec![event])
    }
}

pub struct DoSomethingElse {
    pub description: String,
}

impl Command<TestAggregate, TestEvent> for DoSomethingElse {
    fn handle(self, _aggregate: &TestAggregate) -> Result<Vec<TestEvent>, AggregateError> {
        let event = TestEvent::SomethingElse(SomethingElse { description: self.description.clone() });
        Ok(vec![event])
    }
}

struct TestView {
    events: Rc<RwLock<Vec<MessageEnvelope<TestAggregate, TestEvent>>>>
}

impl TestView {
    fn new(events: Rc<RwLock<Vec<MessageEnvelope<TestAggregate, TestEvent>>>>) -> Self { TestView { events } }
}

impl QueryProcessor<TestAggregate, TestEvent> for TestView {
    fn dispatch(&self, _aggregate_id: &str, events: Vec<MessageEnvelope<TestAggregate, TestEvent>>) {
        for event in events {
            let mut event_list = self.events.write().unwrap();
            event_list.push(event);
        }
    }
}

pub type TestMessageEnvelope = MessageEnvelope<TestAggregate, TestEvent>;

assert_impl_all!(aggregate; TestAggregate,Aggregate);
assert_impl_all!(event; TestEvent,DomainEvent<TestAggregate>);

assert_impl_all!(command_a; CreateTest,Command<TestAggregate,TestEvent>);
assert_impl_all!(command_b; ConfirmTest,Command<TestAggregate,TestEvent>);
assert_impl_all!(command_c; DoSomethingElse,Command<TestAggregate,TestEvent>);

assert_impl_all!(memstore; MemStore::<TestAggregate,TestEvent>, EventStore::<TestAggregate,TestEvent>);

fn metadata() -> HashMap<String, String> {
    let now = Utc::now();
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), now.to_rfc3339());
    metadata
}

#[test]
fn load_events() {
    let event_store = MemStore::<TestAggregate, TestEvent>::default();
    let id = "test_id_A";
    let initial_events = event_store.load(&id);
    assert_eq!(0, initial_events.len());
    let aggregate_type = "TestAggregate".to_string();

    event_store.commit(vec![
        TestMessageEnvelope::new_with_metadata(
            id.to_string(),
            0,
            aggregate_type.clone(),
            TestEvent::Created(Created { id: "test_event_A".to_string() }),
            metadata(),
        )
    ]);
    let stored_events = event_store.load(&id);
    for (i, stored_event) in stored_events.into_iter().enumerate() {
        println!("found event: {}-{:?}", i, stored_event);
    }

    event_store.commit(vec![
        TestMessageEnvelope::new_with_metadata(
            id.to_string(),
            1,
            aggregate_type.clone(),
            TestEvent::Tested(Tested { test_name: "test A".to_string() }),
            metadata()),
        TestMessageEnvelope::new_with_metadata(
            id.to_string(),
            2,
            aggregate_type.clone(),
            TestEvent::Tested(Tested { test_name: "test B".to_string() }),
            metadata()),
        TestMessageEnvelope::new_with_metadata(
            id.to_string(),
            3,
            aggregate_type.clone(),
            TestEvent::SomethingElse(SomethingElse { description: "something else happening here".to_string() }),
            metadata())
    ]);
    let stored_envelopes = event_store.load(&id);

    let mut agg = TestAggregate::default();
    for stored_envelope in stored_envelopes {
        let event = stored_envelope.payload;
        event.apply(&mut agg);
    }
    println!("{:#?}", agg);
}

type ThisTestFramework = TestFramework<TestAggregate, TestEvent>;

#[test]
fn test_framework_test() {
    let test_name = "test A";
    let test_framework = ThisTestFramework::default();

    test_framework.given(vec![TestEvent::Created(Created { id: "test_id_A".to_string() })])
        .when(ConfirmTest { test_name })
        .then_expect_events(vec![TestEvent::Tested(Tested { test_name: test_name.to_string() })]);

    test_framework.given(vec![TestEvent::Tested(Tested { test_name: test_name.to_string() })])
        .when(ConfirmTest { test_name })
        .then_expect_error("test already performed")
}

#[test]
#[should_panic]
fn test_framework_failure_test() {
    let test_name = "test A";
    let test_framework = ThisTestFramework::default();

    test_framework.given(vec![TestEvent::Tested(Tested { test_name: test_name.to_string() })])
        .when(ConfirmTest { test_name })
        .then_expect_events(vec![TestEvent::Tested(Tested { test_name: test_name.to_string() })]);
}

#[test]
#[should_panic]
fn test_framework_failure_test_b() {
    let test_name = "test A";
    let test_framework = ThisTestFramework::default();

    test_framework.given(vec![TestEvent::Created(Created { id: "test_id_A".to_string() })])
        .when(ConfirmTest { test_name })
        .then_expect_error("some error message")
}

#[test]
fn framework_test() {
    let stored_events = Default::default();
    let event_store = MemStore::new_with_shared_events(Rc::clone(&stored_events));

    let delivered_events = Default::default();
    let view = TestView::new(Rc::clone(&delivered_events));

    let cqrs = CqrsFramework::new(event_store, Rc::new(view), TimeMetadataSupplier {});
    let uuid = uuid::Uuid::new_v4().to_string();
    let id = uuid.clone();
    cqrs.execute(&id, CreateTest { id: uuid.clone() }).unwrap_or_default();

    assert_eq!(1, stored_events.read().unwrap().len());
    assert_eq!(1, delivered_events.read().unwrap().len());

    let test = "TEST_A";
    let id = uuid.clone();
    cqrs.execute(&id, ConfirmTest { test_name: test }).unwrap_or_default();

    assert_eq!(2, delivered_events.read().unwrap().len());
    let stored_event_count = stored_events.read().unwrap().get(uuid.clone().as_str()).unwrap().len();
    assert_eq!(2, stored_event_count);

    let id = uuid.clone();
    let err = cqrs.execute(&id, ConfirmTest { test_name: test }).unwrap_err();
    assert_eq!(AggregateError::new("test already performed"), err);

    assert_eq!(2, delivered_events.read().unwrap().len());
    let stored_event_count = stored_events.read().unwrap().get(uuid.clone().as_str()).unwrap().len();
    assert_eq!(2, stored_event_count);
}
