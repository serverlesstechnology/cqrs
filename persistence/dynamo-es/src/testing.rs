#[cfg(test)]
pub(crate) mod tests {
    use std::collections::HashMap;
    use std::fmt::{Display, Formatter};

    use async_trait::async_trait;
    use aws_sdk_dynamodb::config::{Credentials, Region};
    use aws_sdk_dynamodb::Client;
    use cqrs_es::persist::{
        GenericQuery, PersistedEventRepository, PersistedEventStore, SerializedEvent,
        SerializedSnapshot,
    };
    use cqrs_es::{Aggregate, DomainEvent, EventEnvelope, EventStore, View};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    use crate::{DynamoEventRepository, DynamoViewRepository};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub(crate) struct TestAggregate {
        pub(crate) id: String,
        pub(crate) description: String,
        pub(crate) tests: Vec<String>,
    }

    #[async_trait]
    impl Aggregate for TestAggregate {
        type Command = TestCommand;
        type Event = TestEvent;
        type Error = TestError;
        type Services = TestServices;

        fn aggregate_type() -> String {
            "TestAggregate".to_string()
        }

        async fn handle(
            &self,
            _command: Self::Command,
            _services: &Self::Services,
        ) -> Result<Vec<Self::Event>, Self::Error> {
            Ok(vec![])
        }

        fn apply(&mut self, _e: Self::Event) {}
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

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub enum TestEvent {
        Created(Created),
        Tested(Tested),
        SomethingElse(SomethingElse),
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct Created {
        pub id: String,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct Tested {
        pub test_name: String,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct SomethingElse {
        pub description: String,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> String {
            match self {
                TestEvent::Created(_) => "Created".to_string(),
                TestEvent::Tested(_) => "Tested".to_string(),
                TestEvent::SomethingElse(_) => "SomethingElse".to_string(),
            }
        }

        fn event_version(&self) -> String {
            "1.0".to_string()
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct TestError(String);

    pub struct TestServices;

    impl Display for TestError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for TestError {}

    pub enum TestCommand {}

    pub(crate) type TestQueryRepository =
        GenericQuery<DynamoViewRepository<TestView, TestAggregate>, TestView, TestAggregate>;

    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
    pub(crate) struct TestView {
        pub(crate) events: Vec<TestEvent>,
    }

    impl View<TestAggregate> for TestView {
        fn update(&mut self, event: &EventEnvelope<TestAggregate>) {
            self.events.push(event.payload.clone());
        }
    }

    pub async fn test_dynamodb_client() -> Client {
        let region = Region::new("us-west-2");
        let credentials = Credentials::new("TESTAWSID", "TESTAWSKEY", None, None, "");
        let config = aws_sdk_dynamodb::config::Config::builder()
            .behavior_version_latest()
            .region(region)
            .endpoint_url("http://localhost:8000")
            .credentials_provider(credentials)
            .build();
        aws_sdk_dynamodb::client::Client::from_conf(config)
    }

    pub(crate) async fn new_test_event_store(
        client: Client,
    ) -> PersistedEventStore<DynamoEventRepository, TestAggregate> {
        let repo = DynamoEventRepository::new(client);
        PersistedEventStore::<DynamoEventRepository, TestAggregate>::new_event_store(repo)
    }

    pub(crate) fn new_test_metadata() -> HashMap<String, String> {
        let now = "2021-03-18T12:32:45.930Z".to_string();
        let mut metadata = HashMap::new();
        metadata.insert("time".to_string(), now);
        metadata
    }

    pub(crate) fn test_event_envelope(
        id: &str,
        sequence: usize,
        event: TestEvent,
    ) -> SerializedEvent {
        let payload: Value = serde_json::to_value(&event).unwrap();
        SerializedEvent {
            aggregate_id: id.to_string(),
            sequence,
            aggregate_type: TestAggregate::aggregate_type().to_string(),
            event_type: event.event_type().to_string(),
            event_version: event.event_version().to_string(),
            payload,
            metadata: Default::default(),
        }
    }

    pub(crate) fn snapshot_context(
        aggregate_id: String,
        current_sequence: usize,
        current_snapshot: usize,
        aggregate: Value,
    ) -> SerializedSnapshot {
        SerializedSnapshot {
            aggregate_id,
            aggregate,
            current_sequence,
            current_snapshot,
        }
    }

    #[tokio::test]
    async fn commit_and_load_events() {
        let client = test_dynamodb_client().await;
        let event_store = new_test_event_store(client).await;
        let id = uuid::Uuid::new_v4().to_string();
        assert_eq!(0, event_store.load_events(id.as_str()).await.unwrap().len());
        let context = event_store.load_aggregate(id.as_str()).await.unwrap();

        event_store
            .commit(
                vec![
                    TestEvent::Created(Created {
                        id: "test_event_A".to_string(),
                    }),
                    TestEvent::Tested(Tested {
                        test_name: "test A".to_string(),
                    }),
                ],
                context,
                new_test_metadata(),
            )
            .await
            .unwrap();

        assert_eq!(2, event_store.load_events(id.as_str()).await.unwrap().len());
        let context = event_store.load_aggregate(id.as_str()).await.unwrap();

        event_store
            .commit(
                vec![TestEvent::Tested(Tested {
                    test_name: "test B".to_string(),
                })],
                context,
                new_test_metadata(),
            )
            .await
            .unwrap();
        assert_eq!(3, event_store.load_events(id.as_str()).await.unwrap().len());
    }

    #[tokio::test]
    async fn event_repositories() {
        let client = test_dynamodb_client().await;
        let id = uuid::Uuid::new_v4().to_string();
        let event_repo = DynamoEventRepository::new(client.clone());
        let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
        assert!(events.is_empty());

        event_repo
            .insert_events(&[
                test_event_envelope(&id, 1, TestEvent::Created(Created { id: id.clone() })),
                test_event_envelope(
                    &id,
                    2,
                    TestEvent::Tested(Tested {
                        test_name: "a test was run".to_string(),
                    }),
                ),
            ])
            .await
            .unwrap();
        let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
        assert_eq!(2, events.len());
        events.iter().for_each(|e| assert_eq!(&id, &e.aggregate_id));

        event_repo
            .insert_events(&[
                test_event_envelope(
                    &id,
                    3,
                    TestEvent::SomethingElse(SomethingElse {
                        description: "this should not persist".to_string(),
                    }),
                ),
                test_event_envelope(
                    &id,
                    2,
                    TestEvent::SomethingElse(SomethingElse {
                        description: "bad sequence number".to_string(),
                    }),
                ),
            ])
            .await
            .unwrap_err();
        let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
        assert_eq!(2, events.len());
    }
}
