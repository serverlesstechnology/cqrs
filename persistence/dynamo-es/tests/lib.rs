extern crate core;

use aws_sdk_dynamodb::config::{Credentials, Region};
use aws_sdk_dynamodb::primitives::Blob;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use cqrs_es::doc::{Customer, CustomerEvent};
use cqrs_es::persist::{PersistedEventStore, SemanticVersionEventUpcaster};
use cqrs_es::EventStore;
use dynamo_es::DynamoEventRepository;
use serde_json::Value;

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
) -> PersistedEventStore<DynamoEventRepository, Customer> {
    let repo = DynamoEventRepository::new(client);
    PersistedEventStore::<DynamoEventRepository, Customer>::new_event_store(repo)
}

#[tokio::test]
async fn commit_and_load_events() {
    let client = test_dynamodb_client().await;
    let repo = DynamoEventRepository::new(client);
    let event_store = PersistedEventStore::<DynamoEventRepository, Customer>::new_event_store(repo);

    simple_es_commit_and_load_test(event_store).await;
}

#[tokio::test]
async fn commit_and_load_events_snapshot_store() {
    let client = test_dynamodb_client().await;
    let repo = DynamoEventRepository::new(client);
    let event_store =
        PersistedEventStore::<DynamoEventRepository, Customer>::new_aggregate_store(repo);

    simple_es_commit_and_load_test(event_store).await;
}

async fn simple_es_commit_and_load_test(
    event_store: PersistedEventStore<DynamoEventRepository, Customer>,
) {
    let id = uuid::Uuid::new_v4().to_string();
    assert_eq!(0, event_store.load_events(id.as_str()).await.unwrap().len());
    let context = event_store.load_aggregate(id.as_str()).await.unwrap();

    event_store
        .commit(
            vec![
                CustomerEvent::NameAdded {
                    name: "test_event_A".to_string(),
                },
                CustomerEvent::EmailUpdated {
                    new_email: "email A".to_string(),
                },
            ],
            context,
            Default::default(),
        )
        .await
        .unwrap();

    assert_eq!(2, event_store.load_events(id.as_str()).await.unwrap().len());
    let context = event_store.load_aggregate(id.as_str()).await.unwrap();

    event_store
        .commit(
            vec![CustomerEvent::EmailUpdated {
                new_email: "email B".to_string(),
            }],
            context,
            Default::default(),
        )
        .await
        .unwrap();
    assert_eq!(3, event_store.load_events(id.as_str()).await.unwrap().len());
}

#[tokio::test]
async fn commit_no_events() {
    let client = test_dynamodb_client().await;
    let repo = DynamoEventRepository::new(client);
    let event_store = PersistedEventStore::<DynamoEventRepository, Customer>::new_event_store(repo);
    let id = uuid::Uuid::new_v4().to_string();
    let context = event_store.load_aggregate(id.as_str()).await.unwrap();

    event_store
        .commit(vec![], context, Default::default())
        .await
        .unwrap();
}

#[tokio::test]
async fn upcasted_event() {
    let client = test_dynamodb_client().await;
    client
        .put_item()
        .table_name("Events")
        .item(
            "AggregateTypeAndId",
            AttributeValue::S("Customer:previous_event_in_need_of_upcast".to_string()),
        )
        .item("AggregateIdSequence", AttributeValue::N("1".to_string()))
        .item("AggregateType", AttributeValue::S("Customer".to_string()))
        .item(
            "AggregateId",
            AttributeValue::S("previous_event_in_need_of_upcast".to_string()),
        )
        .item("EventVersion", AttributeValue::S("1.0".to_string()))
        .item("EventType", AttributeValue::S("NameAdded".to_string()))
        .item(
            "Payload",
            AttributeValue::B(Blob::new("{\"NameAdded\": {}}".as_bytes())),
        )
        .item("Metadata", AttributeValue::B(Blob::new("{}".as_bytes())))
        .send()
        .await
        .unwrap();

    let upcaster = SemanticVersionEventUpcaster::new(
        "NameAdded",
        "1.0.1",
        Box::new(|mut event| match event.get_mut("NameAdded").unwrap() {
            Value::Object(object) => {
                object.insert("name".to_string(), Value::String("UNKNOWN".to_string()));
                event
            }
            _ => panic!("not the expected object"),
        }),
    );
    let event_store = new_test_event_store(client)
        .await
        .with_upcasters(vec![Box::new(upcaster)]);

    let id = "previous_event_in_need_of_upcast".to_string();
    let result = match event_store.load_aggregate(id.as_str()).await {
        Ok(result) => result,
        Err(err) => panic!("Unexpected error during upcast: {}", err),
    };
    assert_eq!(1, result.current_sequence);
    assert_eq!(None, result.current_snapshot);
}
