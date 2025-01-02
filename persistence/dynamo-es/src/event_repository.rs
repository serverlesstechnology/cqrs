use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::operation::query::builders::QueryFluentBuilder;
use aws_sdk_dynamodb::operation::query::QueryOutput;
use aws_sdk_dynamodb::operation::scan::builders::ScanFluentBuilder;
use aws_sdk_dynamodb::primitives::Blob;
use aws_sdk_dynamodb::types::{AttributeValue, Put, TransactWriteItem};
use aws_sdk_dynamodb::Client;
use cqrs_es::persist::{
    PersistedEventRepository, PersistenceError, ReplayStream, SerializedEvent, SerializedSnapshot,
};
use cqrs_es::Aggregate;
use serde_json::Value;

use crate::error::DynamoAggregateError;
use crate::helpers::{att_as_number, att_as_string, att_as_value, commit_transactions};

const DEFAULT_EVENT_TABLE: &str = "Events";
const DEFAULT_SNAPSHOT_TABLE: &str = "Snapshots";

const DEFAULT_STREAMING_CHANNEL_SIZE: usize = 200;

/// An event repository relying on DynamoDb for persistence.
pub struct DynamoEventRepository {
    client: Client,
    event_table: String,
    snapshot_table: String,
    stream_channel_size: usize,
}

impl DynamoEventRepository {
    /// Creates a new `DynamoEventRepository` from the provided dynamo client using default
    /// table names.
    ///
    /// ```
    /// use aws_sdk_dynamodb::Client;
    /// use dynamo_es::DynamoEventRepository;
    ///
    /// fn configure_repo(client: Client) -> DynamoEventRepository {
    ///     DynamoEventRepository::new(client)
    /// }
    /// ```
    pub fn new(client: Client) -> Self {
        Self::use_table_names(client, DEFAULT_EVENT_TABLE, DEFAULT_SNAPSHOT_TABLE)
    }
    /// Configures a `DynamoEventRepository` to use a streaming queue of the provided size.
    ///
    /// _Example: configure the repository to stream with a 1000 event buffer._
    /// ```
    /// use aws_sdk_dynamodb::Client;
    /// use dynamo_es::DynamoEventRepository;
    ///
    /// fn configure_repo(client: Client) -> DynamoEventRepository {
    ///     let store = DynamoEventRepository::new(client);
    ///     store.with_streaming_channel_size(1000)
    /// }
    /// ```
    pub fn with_streaming_channel_size(self, stream_channel_size: usize) -> Self {
        Self {
            client: self.client,
            event_table: self.event_table,
            snapshot_table: self.snapshot_table,
            stream_channel_size,
        }
    }
    /// Configures a `DynamoEventRepository` to use the provided table names.
    ///
    /// _Example: configure the repository to use "my_event_table" and "my_snapshot_table"
    /// for the event and snapshot table names._
    /// ```
    /// use aws_sdk_dynamodb::Client;
    /// use dynamo_es::DynamoEventRepository;
    ///
    /// fn configure_repo(client: Client) -> DynamoEventRepository {
    ///     let store = DynamoEventRepository::new(client);
    ///     store.with_tables("my_event_table", "my_snapshot_table")
    /// }
    /// ```
    pub fn with_tables(self, event_table: &str, snapshot_table: &str) -> Self {
        Self::use_table_names(self.client, event_table, snapshot_table)
    }

    fn use_table_names(client: Client, event_table: &str, snapshot_table: &str) -> Self {
        Self {
            client,
            event_table: event_table.to_string(),
            snapshot_table: snapshot_table.to_string(),
            stream_channel_size: DEFAULT_STREAMING_CHANNEL_SIZE,
        }
    }

    pub(crate) async fn insert_events(
        &self,
        events: &[SerializedEvent],
    ) -> Result<(), DynamoAggregateError> {
        if events.is_empty() {
            return Ok(());
        }
        let (transactions, _) = Self::build_event_put_transactions(&self.event_table, events);
        commit_transactions(&self.client, transactions).await?;
        Ok(())
    }

    fn build_event_put_transactions(
        table_name: &str,
        events: &[SerializedEvent],
    ) -> (Vec<TransactWriteItem>, usize) {
        let mut current_sequence: usize = 0;
        let mut transactions: Vec<TransactWriteItem> = Vec::default();
        for event in events {
            current_sequence = event.sequence;
            let aggregate_type_and_id =
                AttributeValue::S(format!("{}:{}", &event.aggregate_type, &event.aggregate_id));
            let aggregate_type = AttributeValue::S(String::from(&event.aggregate_type));
            let aggregate_id = AttributeValue::S(String::from(&event.aggregate_id));
            let sequence = AttributeValue::N(String::from(&event.sequence.to_string()));
            let event_version = AttributeValue::S(String::from(&event.event_version));
            let event_type = AttributeValue::S(String::from(&event.event_type));
            let payload_blob = serde_json::to_vec(&event.payload).unwrap();
            let payload = AttributeValue::B(Blob::new(payload_blob));
            let metadata_blob = serde_json::to_vec(&event.metadata).unwrap();
            let metadata = AttributeValue::B(Blob::new(metadata_blob));

            let put = Put::builder()
                .table_name(table_name)
                .item("AggregateTypeAndId", aggregate_type_and_id)
                .item("AggregateIdSequence", sequence)
                .item("AggregateType", aggregate_type)
                .item("AggregateId", aggregate_id)
                .item("EventVersion", event_version)
                .item("EventType", event_type)
                .item("Payload", payload)
                .item("Metadata", metadata)
                .condition_expression("attribute_not_exists( AggregateIdSequence )")
                .build()
                .unwrap();
            let write_item = TransactWriteItem::builder().put(put).build();
            transactions.push(write_item);
        }
        (transactions, current_sequence)
    }

    async fn query_events(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
    ) -> Result<Vec<SerializedEvent>, DynamoAggregateError> {
        let query_output = self
            .query_table(aggregate_type, aggregate_id, &self.event_table)
            .await?;
        let mut result: Vec<SerializedEvent> = Default::default();
        if let Some(entries) = query_output.items {
            for entry in entries {
                result.push(serialized_event(entry)?);
            }
        }
        Ok(result)
    }
    async fn query_events_from(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
        last_sequence: usize,
    ) -> Result<Vec<SerializedEvent>, DynamoAggregateError> {
        let query_output = self
            .client
            .query()
            .table_name(&self.event_table)
            .key_condition_expression("#agg_type_id = :agg_type_id AND #sequence > :sequence")
            .expression_attribute_names("#agg_type_id", "AggregateTypeAndId")
            .expression_attribute_names("#sequence", "AggregateIdSequence")
            .expression_attribute_values(
                ":agg_type_id",
                AttributeValue::S(format!("{}:{}", aggregate_type, aggregate_id)),
            )
            .expression_attribute_values(":sequence", AttributeValue::N(last_sequence.to_string()))
            .send()
            .await?;
        let mut result: Vec<SerializedEvent> = Default::default();
        if let Some(entries) = query_output.items {
            for entry in entries {
                result.push(serialized_event(entry)?);
            }
        }
        Ok(result)
    }

    pub(crate) async fn update_snapshot<A: Aggregate>(
        &self,
        aggregate_payload: Value,
        aggregate_id: String,
        current_snapshot: usize,
        events: &[SerializedEvent],
    ) -> Result<(), DynamoAggregateError> {
        let expected_snapshot = current_snapshot - 1;
        let (mut transactions, current_sequence) =
            Self::build_event_put_transactions(&self.event_table, events);
        let aggregate_type_and_id =
            AttributeValue::S(format!("{}:{}", A::aggregate_type(), &aggregate_id));
        let aggregate_type = AttributeValue::S(A::aggregate_type());
        let aggregate_id = AttributeValue::S(aggregate_id);
        let current_sequence = AttributeValue::N(current_sequence.to_string());
        let current_snapshot = AttributeValue::N(current_snapshot.to_string());
        let payload_blob = serde_json::to_vec(&aggregate_payload).unwrap();
        let payload = AttributeValue::B(Blob::new(payload_blob));
        let expected_snapshot = AttributeValue::N(expected_snapshot.to_string());
        transactions.push(TransactWriteItem::builder()
            .put(Put::builder()
                .table_name(&self.snapshot_table)
                .item("AggregateTypeAndId", aggregate_type_and_id)
                .item("AggregateType", aggregate_type)
                .item("AggregateId", aggregate_id)
                .item("CurrentSequence", current_sequence)
                .item("CurrentSnapshot", current_snapshot)
                .item("Payload", payload)
                .condition_expression("attribute_not_exists(CurrentSnapshot) OR (CurrentSnapshot  = :current_snapshot)")
                .expression_attribute_values(":current_snapshot", expected_snapshot)
                .build()?)
            .build());
        commit_transactions(&self.client, transactions).await?;
        Ok(())
    }

    async fn query_table(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
        table: &str,
    ) -> Result<QueryOutput, DynamoAggregateError> {
        let query = self.create_query(table, aggregate_type, aggregate_id).await;
        Ok(query.send().await?)
    }

    async fn create_query(
        &self,
        table: &str,
        aggregate_type: &str,
        aggregate_id: &str,
    ) -> QueryFluentBuilder {
        self.client
            .query()
            .table_name(table)
            .consistent_read(true)
            .key_condition_expression("#agg_type_id = :agg_type_id")
            .expression_attribute_names("#agg_type_id", "AggregateTypeAndId")
            .expression_attribute_values(
                ":agg_type_id",
                AttributeValue::S(format!("{}:{}", aggregate_type, aggregate_id)),
            )
    }
}

fn serialized_event(
    entry: HashMap<String, AttributeValue>,
) -> Result<SerializedEvent, DynamoAggregateError> {
    let aggregate_id = att_as_string(&entry, "AggregateId")?;
    let sequence = att_as_number(&entry, "AggregateIdSequence")?;
    let aggregate_type = att_as_string(&entry, "AggregateType")?;
    let event_type = att_as_string(&entry, "EventType")?;
    let event_version = att_as_string(&entry, "EventVersion")?;
    let payload = att_as_value(&entry, "Payload")?;
    let metadata = att_as_value(&entry, "Metadata")?;
    Ok(SerializedEvent {
        aggregate_id,
        sequence,
        aggregate_type,
        event_type,
        event_version,
        payload,
        metadata,
    })
}

#[async_trait]
impl PersistedEventRepository for DynamoEventRepository {
    async fn get_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        let request = self
            .query_events(&A::aggregate_type(), aggregate_id)
            .await?;
        Ok(request)
    }

    async fn get_last_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
        number_events: usize,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        Ok(self
            .query_events_from(&A::aggregate_type(), aggregate_id, number_events)
            .await?)
    }

    async fn get_snapshot<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<Option<SerializedSnapshot>, PersistenceError> {
        let query_output = self
            .query_table(&A::aggregate_type(), aggregate_id, &self.snapshot_table)
            .await?;
        let query_items_vec = match query_output.items {
            None => return Ok(None),
            Some(items) => items,
        };
        if query_items_vec.is_empty() {
            return Ok(None);
        }
        let query_item = query_items_vec.first().unwrap();
        let aggregate = att_as_value(query_item, "Payload")?;
        let current_sequence = att_as_number(query_item, "CurrentSequence")?;
        let current_snapshot = att_as_number(query_item, "CurrentSnapshot")?;

        Ok(Some(SerializedSnapshot {
            aggregate_id: aggregate_id.to_string(),
            aggregate,
            current_sequence,
            current_snapshot,
        }))
    }

    async fn persist<A: Aggregate>(
        &self,
        events: &[SerializedEvent],
        snapshot_update: Option<(String, Value, usize)>,
    ) -> Result<(), PersistenceError> {
        match snapshot_update {
            None => {
                self.insert_events(events).await?;
            }
            Some((aggregate_id, aggregate, current_snapshot)) => {
                self.update_snapshot::<A>(aggregate, aggregate_id, current_snapshot, events)
                    .await?;
            }
        }
        Ok(())
    }

    async fn stream_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<ReplayStream, PersistenceError> {
        let query = self
            .create_query(&self.event_table, &A::aggregate_type(), aggregate_id)
            .await
            .limit(self.stream_channel_size as i32);
        Ok(stream_events(query, self.stream_channel_size))
    }

    async fn stream_all_events<A: Aggregate>(&self) -> Result<ReplayStream, PersistenceError> {
        let scan = self
            .client
            .scan()
            .table_name(&self.event_table)
            .limit(self.stream_channel_size as i32);
        Ok(stream_all_events(scan, self.stream_channel_size))
    }
}

// TODO: combine these two methods
fn stream_events(base_query: QueryFluentBuilder, channel_size: usize) -> ReplayStream {
    let (mut feed, stream) = ReplayStream::new(channel_size);
    tokio::spawn(async move {
        let mut last_evaluated_key: Option<HashMap<String, AttributeValue>> = None;
        loop {
            let query = match &last_evaluated_key {
                None => base_query.clone(),
                Some(last) => {
                    let mut query = base_query.clone();
                    for (key, value) in last {
                        query = query.exclusive_start_key(key.to_string(), value.to_owned());
                    }
                    query
                }
            };
            match query.send().await {
                Ok(query_output) => {
                    last_evaluated_key = query_output.last_evaluated_key;
                    if let Some(entries) = query_output.items {
                        for entry in entries {
                            let event = match serialized_event(entry) {
                                Ok(event) => event,
                                Err(_) => return,
                            };
                            if feed.push(Ok(event)).await.is_err() {
                                //         TODO: in the unlikely event of a broken channel this error should be reported.
                                return;
                            };
                        }
                    };
                }
                Err(err) => {
                    let err: DynamoAggregateError = err.into();
                    if feed.push(Err(err.into())).await.is_err() {};
                }
            }
            if last_evaluated_key.is_none() {
                return;
            }
        }
    });
    stream
}
fn stream_all_events(base_query: ScanFluentBuilder, channel_size: usize) -> ReplayStream {
    let (mut feed, stream) = ReplayStream::new(channel_size);
    tokio::spawn(async move {
        let mut last_evaluated_key: Option<HashMap<String, AttributeValue>> = None;
        loop {
            let query = match &last_evaluated_key {
                None => base_query.clone(),
                Some(last) => {
                    let mut query = base_query.clone();
                    for (key, value) in last {
                        query = query.exclusive_start_key(key.to_string(), value.to_owned());
                    }
                    query
                }
            };
            match query.send().await {
                Ok(query_output) => {
                    last_evaluated_key = query_output.last_evaluated_key;
                    if let Some(entries) = query_output.items {
                        for entry in entries {
                            let event = match serialized_event(entry) {
                                Ok(event) => event,
                                Err(_) => return,
                            };
                            if feed.push(Ok(event)).await.is_err() {
                                //         TODO: in the unlikely event of a broken channel this error should be reported.
                                return;
                            };
                        }
                    };
                }
                Err(err) => {
                    let err: DynamoAggregateError = err.into();
                    if feed.push(Err(err.into())).await.is_err() {};
                }
            }
            if last_evaluated_key.is_none() {
                return;
            }
        }
    });
    stream
}

#[cfg(test)]
mod test {
    use cqrs_es::persist::PersistedEventRepository;

    use crate::error::DynamoAggregateError;
    use crate::testing::tests::{
        snapshot_context, test_dynamodb_client, test_event_envelope, Created, SomethingElse,
        TestAggregate, TestEvent, Tested,
    };
    use crate::DynamoEventRepository;

    #[tokio::test]
    async fn event_repositories() {
        let client = test_dynamodb_client().await;
        let id = uuid::Uuid::new_v4().to_string();
        let event_repo = DynamoEventRepository::new(client.clone()).with_streaming_channel_size(1);
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

        // Optimistic lock error
        let result = event_repo
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
        match result {
            DynamoAggregateError::OptimisticLock => {}
            _ => panic!("invalid error result found during insert: {}", result),
        };

        let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
        assert_eq!(2, events.len());

        let events = event_repo
            .get_last_events::<TestAggregate>(&id, 1)
            .await
            .unwrap();
        assert_eq!(1, events.len());

        verify_replay_stream(&id, event_repo).await;
    }

    async fn verify_replay_stream(id: &str, event_repo: DynamoEventRepository) {
        let mut stream = event_repo
            .stream_events::<TestAggregate>(&id)
            .await
            .unwrap();
        let mut found_in_stream = 0;
        while let Some(_) = stream.next::<TestAggregate>(&None).await {
            found_in_stream += 1;
        }
        assert_eq!(found_in_stream, 2);

        let mut stream = event_repo
            .stream_all_events::<TestAggregate>()
            .await
            .unwrap();
        let mut found_in_stream = 0;
        while let Some(_) = stream.next::<TestAggregate>(&None).await {
            found_in_stream += 1;
        }
        assert!(found_in_stream >= 2);
    }

    #[tokio::test]
    async fn snapshot_repositories() {
        let client = test_dynamodb_client().await;
        let id = uuid::Uuid::new_v4().to_string();
        let repo = DynamoEventRepository::new(client.clone());
        let snapshot = repo.get_snapshot::<TestAggregate>(&id).await.unwrap();
        assert_eq!(None, snapshot);

        let test_description = "some test snapshot here".to_string();
        let test_tests = vec!["testA".to_string(), "testB".to_string()];
        repo.update_snapshot::<TestAggregate>(
            serde_json::to_value(TestAggregate {
                id: id.clone(),
                description: test_description.clone(),
                tests: test_tests.clone(),
            })
            .unwrap(),
            id.clone(),
            1,
            &vec![],
        )
        .await
        .unwrap();

        let snapshot = repo.get_snapshot::<TestAggregate>(&id).await.unwrap();
        assert_eq!(
            Some(snapshot_context(
                id.clone(),
                0,
                1,
                serde_json::to_value(TestAggregate {
                    id: id.clone(),
                    description: test_description.clone(),
                    tests: test_tests.clone(),
                })
                .unwrap(),
            )),
            snapshot
        );

        // sequence iterated, does update
        repo.update_snapshot::<TestAggregate>(
            serde_json::to_value(TestAggregate {
                id: id.clone(),
                description: "a test description that should be saved".to_string(),
                tests: test_tests.clone(),
            })
            .unwrap(),
            id.clone(),
            2,
            &vec![],
        )
        .await
        .unwrap();

        let snapshot = repo.get_snapshot::<TestAggregate>(&id).await.unwrap();
        assert_eq!(
            Some(snapshot_context(
                id.clone(),
                0,
                2,
                serde_json::to_value(TestAggregate {
                    id: id.clone(),
                    description: "a test description that should be saved".to_string(),
                    tests: test_tests.clone(),
                })
                .unwrap(),
            )),
            snapshot
        );

        // sequence out of order or not iterated, does not update
        let result = repo
            .update_snapshot::<TestAggregate>(
                serde_json::to_value(TestAggregate {
                    id: id.clone(),
                    description: "a test description that should not be saved".to_string(),
                    tests: test_tests.clone(),
                })
                .unwrap(),
                id.clone(),
                2,
                &vec![],
            )
            .await
            .unwrap_err();
        match result {
            DynamoAggregateError::OptimisticLock => {}
            _ => panic!("invalid error result found during insert: {}", result),
        };

        let snapshot = repo.get_snapshot::<TestAggregate>(&id).await.unwrap();
        assert_eq!(
            Some(snapshot_context(
                id.clone(),
                0,
                2,
                serde_json::to_value(TestAggregate {
                    id: id.clone(),
                    description: "a test description that should be saved".to_string(),
                    tests: test_tests.clone(),
                })
                .unwrap(),
            )),
            snapshot
        );
    }
}
