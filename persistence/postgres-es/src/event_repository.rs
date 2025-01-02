use async_trait::async_trait;
use cqrs_es::persist::{
    PersistedEventRepository, PersistenceError, ReplayStream, SerializedEvent, SerializedSnapshot,
};
use cqrs_es::Aggregate;
use futures::TryStreamExt;
use serde_json::Value;
use sqlx::postgres::PgRow;
use sqlx::{Pool, Postgres, Row, Transaction};

use crate::error::PostgresAggregateError;
use crate::sql_query::SqlQueryFactory;

const DEFAULT_EVENT_TABLE: &str = "events";
const DEFAULT_SNAPSHOT_TABLE: &str = "snapshots";

const DEFAULT_STREAMING_CHANNEL_SIZE: usize = 200;

/// An event repository relying on a Postgres database for persistence.
pub struct PostgresEventRepository {
    pool: Pool<Postgres>,
    query_factory: SqlQueryFactory,
    stream_channel_size: usize,
}

#[async_trait]
impl PersistedEventRepository for PostgresEventRepository {
    async fn get_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        self.select_events::<A>(aggregate_id, self.query_factory.select_events())
            .await
    }

    async fn get_last_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
        last_sequence: usize,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        let query = self.query_factory.get_last_events(last_sequence);
        self.select_events::<A>(aggregate_id, &query).await
    }

    async fn get_snapshot<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<Option<SerializedSnapshot>, PersistenceError> {
        let row: PgRow = match sqlx::query(self.query_factory.select_snapshot())
            .bind(A::aggregate_type())
            .bind(aggregate_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(PostgresAggregateError::from)?
        {
            Some(row) => row,
            None => {
                return Ok(None);
            }
        };
        Ok(Some(self.deser_snapshot(row)?))
    }

    async fn persist<A: Aggregate>(
        &self,
        events: &[SerializedEvent],
        snapshot_update: Option<(String, Value, usize)>,
    ) -> Result<(), PersistenceError> {
        match snapshot_update {
            None => {
                self.insert_events::<A>(events).await?;
            }
            Some((aggregate_id, aggregate, current_snapshot)) => {
                if current_snapshot == 1 {
                    self.insert::<A>(aggregate, aggregate_id, current_snapshot, events)
                        .await?;
                } else {
                    self.update::<A>(aggregate, aggregate_id, current_snapshot, events)
                        .await?;
                }
            }
        };
        Ok(())
    }

    async fn stream_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<ReplayStream, PersistenceError> {
        Ok(stream_events(
            self.query_factory.select_events().to_string(),
            A::aggregate_type(),
            aggregate_id.to_string(),
            self.pool.clone(),
            self.stream_channel_size,
        ))
    }

    // TODO: aggregate id is unused here, `stream_events` function needs to be broken up
    async fn stream_all_events<A: Aggregate>(&self) -> Result<ReplayStream, PersistenceError> {
        Ok(stream_events(
            self.query_factory.all_events().to_string(),
            A::aggregate_type(),
            "".to_string(),
            self.pool.clone(),
            self.stream_channel_size,
        ))
    }
}

fn stream_events(
    query: String,
    aggregate_type: String,
    aggregate_id: String,
    pool: Pool<Postgres>,
    channel_size: usize,
) -> ReplayStream {
    let (mut feed, stream) = ReplayStream::new(channel_size);
    tokio::spawn(async move {
        let query = sqlx::query(&query)
            .bind(&aggregate_type)
            .bind(&aggregate_id);
        let mut rows = query.fetch(&pool);
        while let Some(row) = rows.try_next().await.unwrap() {
            let event_result: Result<SerializedEvent, PersistenceError> =
                PostgresEventRepository::deser_event(row).map_err(Into::into);
            if feed.push(event_result).await.is_err() {
                // TODO: in the unlikely event of a broken channel this error should be reported.
                return;
            };
        }
    });
    stream
}

impl PostgresEventRepository {
    async fn select_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
        query: &str,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        let mut rows = sqlx::query(query)
            .bind(A::aggregate_type())
            .bind(aggregate_id)
            .fetch(&self.pool);
        let mut result: Vec<SerializedEvent> = Default::default();
        while let Some(row) = rows
            .try_next()
            .await
            .map_err(PostgresAggregateError::from)?
        {
            result.push(PostgresEventRepository::deser_event(row)?);
        }
        Ok(result)
    }
}

impl PostgresEventRepository {
    /// Creates a new `PostgresEventRepository` from the provided database connection.
    /// This uses the default tables 'events' and 'snapshots'.
    ///
    /// ```
    /// use sqlx::{Pool, Postgres};
    /// use postgres_es::PostgresEventRepository;
    ///
    /// fn configure_repo(pool: Pool<Postgres>) -> PostgresEventRepository {
    ///     PostgresEventRepository::new(pool)
    /// }
    /// ```
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self::use_tables(pool, DEFAULT_EVENT_TABLE, DEFAULT_SNAPSHOT_TABLE)
    }

    /// Configures a `PostgresEventRepository` to use a streaming queue of the provided size.
    ///
    /// _Example: configure the repository to stream with a 1000 event buffer._
    /// ```
    /// use sqlx::{Pool, Postgres};
    /// use postgres_es::PostgresEventRepository;
    ///
    /// fn configure_repo(pool: Pool<Postgres>) -> PostgresEventRepository {
    ///     let store = PostgresEventRepository::new(pool);
    ///     store.with_streaming_channel_size(1000)
    /// }
    /// ```
    pub fn with_streaming_channel_size(self, stream_channel_size: usize) -> Self {
        Self {
            pool: self.pool,
            query_factory: self.query_factory,
            stream_channel_size,
        }
    }

    /// Configures a `PostgresEventRepository` to use the provided table names.
    ///
    /// _Example: configure the repository to use "my_event_table" and "my_snapshot_table"
    /// for the event and snapshot table names._
    /// ```
    /// use sqlx::{Pool, Postgres};
    /// use postgres_es::PostgresEventRepository;
    ///
    /// fn configure_repo(pool: Pool<Postgres>) -> PostgresEventRepository {
    ///     let store = PostgresEventRepository::new(pool);
    ///     store.with_tables("my_event_table", "my_snapshot_table")
    /// }
    /// ```
    pub fn with_tables(self, events_table: &str, snapshots_table: &str) -> Self {
        Self::use_tables(self.pool, events_table, snapshots_table)
    }

    fn use_tables(pool: Pool<Postgres>, events_table: &str, snapshots_table: &str) -> Self {
        Self {
            pool,
            query_factory: SqlQueryFactory::new(events_table, snapshots_table),
            stream_channel_size: DEFAULT_STREAMING_CHANNEL_SIZE,
        }
    }

    pub(crate) async fn insert_events<A: Aggregate>(
        &self,
        events: &[SerializedEvent],
    ) -> Result<(), PostgresAggregateError> {
        let mut tx: Transaction<'_, Postgres> = sqlx::Acquire::begin(&self.pool).await?;
        self.persist_events::<A>(self.query_factory.insert_event(), &mut tx, events)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub(crate) async fn insert<A: Aggregate>(
        &self,
        aggregate_payload: Value,
        aggregate_id: String,
        current_snapshot: usize,
        events: &[SerializedEvent],
    ) -> Result<(), PostgresAggregateError> {
        let mut tx: Transaction<'_, Postgres> = sqlx::Acquire::begin(&self.pool).await?;
        let current_sequence = self
            .persist_events::<A>(self.query_factory.insert_event(), &mut tx, events)
            .await?;
        sqlx::query(self.query_factory.insert_snapshot())
            .bind(A::aggregate_type())
            .bind(aggregate_id.as_str())
            .bind(current_sequence as i32)
            .bind(current_snapshot as i32)
            .bind(&aggregate_payload)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub(crate) async fn update<A: Aggregate>(
        &self,
        aggregate: Value,
        aggregate_id: String,
        current_snapshot: usize,
        events: &[SerializedEvent],
    ) -> Result<(), PostgresAggregateError> {
        let mut tx: Transaction<'_, Postgres> = sqlx::Acquire::begin(&self.pool).await?;
        let current_sequence = self
            .persist_events::<A>(self.query_factory.insert_event(), &mut tx, events)
            .await?;

        let aggregate_payload = serde_json::to_value(&aggregate)?;
        let result = sqlx::query(self.query_factory.update_snapshot())
            .bind(A::aggregate_type())
            .bind(aggregate_id.as_str())
            .bind(current_sequence as i32)
            .bind(current_snapshot as i32)
            .bind((current_snapshot - 1) as i32)
            .bind(&aggregate_payload)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        match result.rows_affected() {
            1 => Ok(()),
            _ => Err(PostgresAggregateError::OptimisticLock),
        }
    }

    fn deser_event(row: PgRow) -> Result<SerializedEvent, PostgresAggregateError> {
        let aggregate_type: String = row.get("aggregate_type");
        let aggregate_id: String = row.get("aggregate_id");
        let sequence = {
            let s: i64 = row.get("sequence");
            s as usize
        };
        let event_type: String = row.get("event_type");
        let event_version: String = row.get("event_version");
        let payload: Value = row.get("payload");
        let metadata: Value = row.get("metadata");
        Ok(SerializedEvent::new(
            aggregate_id,
            sequence,
            aggregate_type,
            event_type,
            event_version,
            payload,
            metadata,
        ))
    }

    fn deser_snapshot(&self, row: PgRow) -> Result<SerializedSnapshot, PostgresAggregateError> {
        let aggregate_id = row.get("aggregate_id");
        let s: i64 = row.get("last_sequence");
        let current_sequence = s as usize;
        let s: i64 = row.get("current_snapshot");
        let current_snapshot = s as usize;
        let aggregate: Value = row.get("payload");
        Ok(SerializedSnapshot {
            aggregate_id,
            aggregate,
            current_sequence,
            current_snapshot,
        })
    }

    pub(crate) async fn persist_events<A: Aggregate>(
        &self,
        inser_event_query: &str,
        tx: &mut Transaction<'_, Postgres>,
        events: &[SerializedEvent],
    ) -> Result<usize, PostgresAggregateError> {
        let mut current_sequence: usize = 0;
        for event in events {
            current_sequence = event.sequence;
            let event_type = &event.event_type;
            let event_version = &event.event_version;
            let payload = serde_json::to_value(&event.payload)?;
            let metadata = serde_json::to_value(&event.metadata)?;
            sqlx::query(inser_event_query)
                .bind(A::aggregate_type())
                .bind(event.aggregate_id.as_str())
                .bind(event.sequence as i32)
                .bind(event_type)
                .bind(event_version)
                .bind(&payload)
                .bind(&metadata)
                .execute(&mut **tx)
                .await?;
        }
        Ok(current_sequence)
    }
}

#[cfg(test)]
mod test {
    use cqrs_es::persist::PersistedEventRepository;

    use crate::error::PostgresAggregateError;
    use crate::testing::tests::{
        snapshot_context, test_event_envelope, Created, SomethingElse, TestAggregate, TestEvent,
        Tested, TEST_CONNECTION_STRING,
    };
    use crate::{default_postgress_pool, PostgresEventRepository};

    #[tokio::test]
    async fn event_repositories() {
        let pool = default_postgress_pool(TEST_CONNECTION_STRING).await;
        let id = uuid::Uuid::new_v4().to_string();
        let event_repo: PostgresEventRepository =
            PostgresEventRepository::new(pool.clone()).with_streaming_channel_size(1);
        let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
        assert!(events.is_empty());

        event_repo
            .insert_events::<TestAggregate>(&[
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
            .insert_events::<TestAggregate>(&[
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
            PostgresAggregateError::OptimisticLock => {}
            _ => panic!("invalid error result found during insert: {}", result),
        };

        let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
        assert_eq!(2, events.len());

        verify_replay_stream(&id, event_repo).await;
    }

    async fn verify_replay_stream(id: &str, event_repo: PostgresEventRepository) {
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
        let pool = default_postgress_pool(TEST_CONNECTION_STRING).await;
        let id = uuid::Uuid::new_v4().to_string();
        let event_repo: PostgresEventRepository = PostgresEventRepository::new(pool.clone());
        let snapshot = event_repo.get_snapshot::<TestAggregate>(&id).await.unwrap();
        assert_eq!(None, snapshot);

        let test_description = "some test snapshot here".to_string();
        let test_tests = vec!["testA".to_string(), "testB".to_string()];
        event_repo
            .insert::<TestAggregate>(
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

        let snapshot = event_repo.get_snapshot::<TestAggregate>(&id).await.unwrap();
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
                .unwrap()
            )),
            snapshot
        );

        // sequence iterated, does update
        event_repo
            .update::<TestAggregate>(
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

        let snapshot = event_repo.get_snapshot::<TestAggregate>(&id).await.unwrap();
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
                .unwrap()
            )),
            snapshot
        );

        // sequence out of order or not iterated, does not update
        let result = event_repo
            .update::<TestAggregate>(
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
            PostgresAggregateError::OptimisticLock => {}
            _ => panic!("invalid error result found during insert: {}", result),
        };

        let snapshot = event_repo.get_snapshot::<TestAggregate>(&id).await.unwrap();
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
                .unwrap()
            )),
            snapshot
        );
    }
}
