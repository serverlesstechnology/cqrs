// /// Storage engine using a database backing.
// /// This is an snapshot-sourced `EventStore`, meaning it uses the serialized aggregate as the
// /// primary source of truth for the state of the aggregate.
// ///
// /// The individual events are also persisted but are used only for updating queries.
// ///
// /// For a event-sourced `EventStore` see [`PersistedEventStore`](struct.PersistedEventStore.html).
// ///
// pub struct PersistedSnapshotStore<R, A>
// where
//     R: PersistedEventRepository,
//     A: Aggregate + Send + Sync,
// {
//     repo: R,
//     event_upcasters: Option<Vec<Box<dyn EventUpcaster>>>,
//     _phantom: PhantomData<A>,
// }

// impl<R, A> PersistedSnapshotStore<R, A>
// where
//     R: PersistedEventRepository,
//     A: Aggregate + Send + Sync,
// {
//     /// Creates a new `PostgresSnapshotStore` from the provided database connection pool,
//     /// an `EventStore` used for configuring a new cqrs framework.
//     ///
//     /// ```
//     /// # use cqrs_es::doc::MyAggregate;
//     /// # use cqrs_es::CqrsFramework;
//     /// # use cqrs_es::persist::doc::{MyDatabaseConnection, MyEventRepository};
//     /// # use cqrs_es::persist::PersistedSnapshotStore;
//     /// # fn config(my_db_connection: MyDatabaseConnection) {
//     /// let repo = MyEventRepository::new(my_db_connection);
//     /// let store = PersistedSnapshotStore::<MyEventRepository,MyAggregate>::new(repo);
//     /// let cqrs = CqrsFramework::new(store, vec![]);
//     /// # }
//     /// ```
//     pub fn new(snapshot_repo: R) -> Self {
//         PersistedSnapshotStore {
//             repo: snapshot_repo,
//             event_upcasters: None,
//             _phantom: PhantomData,
//         }
//     }
// }

// #[async_trait]
// impl<R, A> EventStore<A> for PersistedSnapshotStore<R, A>
// where
//     R: PersistedEventRepository,
//     A: Aggregate + Send + Sync,
// {
//     type AC = SnapshotStoreAggregateContext<A>;
//
//     async fn load_events(
//         &self,
//         aggregate_id: &str,
//     ) -> Result<Vec<EventEnvelope<A>>, AggregateError<A::Error>> {
//         PersistedEventStore::load_from_repo(&self.repo, aggregate_id, &self.event_upcasters).await
//     }
//
//     async fn load_aggregate(
//         &self,
//         aggregate_id: &str,
//     ) -> Result<SnapshotStoreAggregateContext<A>, AggregateError<A::Error>> {
//         let snapshot = self.repo.get_snapshot::<A>(aggregate_id).await?;
//         match snapshot {
//             Some(snapshot) => Ok(snapshot.try_into()?),
//             None => Ok(SnapshotStoreAggregateContext::new(aggregate_id)),
//         }
//     }
//
//     async fn commit(
//         &self,
//         events: Vec<A::Event>,
//         mut context: SnapshotStoreAggregateContext<A>,
//         metadata: HashMap<String, String>,
//     ) -> Result<Vec<EventEnvelope<A>>, AggregateError<A::Error>> {
//         for event in events.clone() {
//             context.aggregate.apply(event);
//         }
//         let aggregate_id = context.aggregate_id.clone();
//         let next_snapshot = context.current_snapshot + 1;
//         let seq = context.current_sequence;
//         let wrapped_events = self.wrap_events(&aggregate_id, seq, events, metadata);
//         let serialized_events = serialize_events(&wrapped_events)?;
//         let payload = serde_json::to_value(context.aggregate)?;
//         let snapshot_update = Some((aggregate_id, payload, next_snapshot));
//         self.repo
//             .persist::<A>(&serialized_events, snapshot_update)
//             .await?;
//         Ok(wrapped_events)
//     }
// }

#[cfg(test)]
pub(crate) mod test {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use crate::{Aggregate, AggregateError, DomainEvent, EventStore, UserErrorPayload};
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use serde_json::Value;

    use crate::persist::event_store::SourceOfTruth;
    use crate::persist::{
        EventStoreAggregateContext, PersistedEventRepository, PersistedEventStore,
        PersistenceError, SerializedEvent, SerializedSnapshot,
    };

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    pub(crate) enum TestEvents {
        Started,
        SomethingWasDone,
    }

    impl DomainEvent for TestEvents {
        fn event_type(&self) -> String {
            match self {
                TestEvents::Started => "Started".to_string(),
                TestEvents::SomethingWasDone => "SomethingWasDone".to_string(),
            }
        }
        fn event_version(&self) -> String {
            EVENT_VERSION.to_string()
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) enum TestCommands {
        DoSomething,
        BadCommand,
    }

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    pub(crate) struct TestAggregate {
        pub(crate) something_happened: usize,
    }

    #[async_trait]
    impl Aggregate for TestAggregate {
        type Command = TestCommands;
        type Event = TestEvents;
        type Error = UserErrorPayload;

        fn aggregate_type() -> String {
            "TestAggregate".to_string()
        }
        async fn handle(
            &self,
            command: Self::Command,
        ) -> Result<Vec<Self::Event>, AggregateError<Self::Error>> {
            match command {
                TestCommands::DoSomething => Ok(vec![TestEvents::SomethingWasDone]),
                TestCommands::BadCommand => Err("the expected error message".into()),
            }
        }
        fn apply(&mut self, event: Self::Event) {
            match event {
                TestEvents::Started => {}
                TestEvents::SomethingWasDone => {
                    self.something_happened += 1;
                }
            }
        }
    }

    pub(crate) struct MockRepo {
        events_result: Mutex<Option<Result<Vec<SerializedEvent>, PersistenceError>>>,
        snapshot_result: Mutex<Option<Result<Option<SerializedSnapshot>, PersistenceError>>>,
        persist_check: Mutex<
            Option<Box<dyn FnOnce(&[SerializedEvent], Option<(String, Value, usize)>) + Send>>,
        >,
    }

    impl MockRepo {
        pub(crate) fn with_events(result: Result<Vec<SerializedEvent>, PersistenceError>) -> Self {
            Self {
                events_result: Mutex::new(Some(result)),
                snapshot_result: Mutex::new(None),
                persist_check: Mutex::new(None),
            }
        }
        pub(crate) fn with_snapshot(
            result: Result<Option<SerializedSnapshot>, PersistenceError>,
        ) -> Self {
            Self {
                events_result: Mutex::new(None),
                snapshot_result: Mutex::new(Some(result)),
                persist_check: Mutex::new(None),
            }
        }
        pub(crate) fn with_commit(
            test_function: Box<
                dyn FnOnce(&[SerializedEvent], Option<(String, Value, usize)>) + Send,
            >,
        ) -> Self {
            Self {
                events_result: Mutex::new(None),
                snapshot_result: Mutex::new(None),
                persist_check: Mutex::new(Some(test_function)),
            }
        }
    }

    #[async_trait]
    impl PersistedEventRepository for MockRepo {
        async fn get_events<A: Aggregate>(
            &self,
            _aggregate_id: &str,
        ) -> Result<Vec<SerializedEvent>, PersistenceError> {
            self.events_result.lock().unwrap().take().unwrap()
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
            self.snapshot_result.lock().unwrap().take().unwrap()
        }
        async fn persist<A: Aggregate>(
            &self,
            events: &[SerializedEvent],
            snapshot_update: Option<(String, Value, usize)>,
        ) -> Result<(), PersistenceError> {
            let test = self.persist_check.lock().unwrap().take().unwrap();
            test(events, snapshot_update);
            Ok(())
        }
    }

    pub(crate) const TEST_AGGREGATE_ID: &str = "test-aggregate-C";
    pub(crate) const EVENT_VERSION: &'static str = "1.0";

    #[tokio::test]
    async fn load() {
        let repo = MockRepo::with_events(Ok(vec![test_serialized_event(
            1,
            TestEvents::SomethingWasDone,
        )]));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new(repo)
            .with_storage_method(SourceOfTruth::AggregateStore);
        let events = store.load_events(TEST_AGGREGATE_ID).await.unwrap();
        let event = events.get(0).unwrap();
        assert_eq!(1, event.sequence);
        assert_eq!("SomethingWasDone", event.payload.event_type());
        assert_eq!(EVENT_VERSION, event.payload.event_version());
    }

    #[tokio::test]
    async fn load_error() {
        let repo = MockRepo::with_events(Err(PersistenceError::OptimisticLockError));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new(repo)
            .with_storage_method(SourceOfTruth::AggregateStore);
        let result = store.load_events(TEST_AGGREGATE_ID).await.unwrap_err();
        match result {
            AggregateError::AggregateConflict => {}
            _ => panic!("expected technical error"),
        }
    }

    #[tokio::test]
    async fn load_aggregate_new() {
        let repo = MockRepo::with_snapshot(Ok(None));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new(repo)
            .with_storage_method(SourceOfTruth::AggregateStore);
        let snapshot_context = store.load_aggregate(TEST_AGGREGATE_ID).await.unwrap();
        assert_eq!(Some(0), snapshot_context.current_snapshot);
        assert_eq!(0, snapshot_context.current_sequence);
        assert_eq!(TEST_AGGREGATE_ID, snapshot_context.aggregate_id);
        assert_eq!(TestAggregate::default(), snapshot_context.aggregate);
    }

    #[tokio::test]
    async fn load_aggregate_existing() {
        let repo = MockRepo::with_snapshot(Ok(Some(SerializedSnapshot {
            aggregate_id: TEST_AGGREGATE_ID.to_string(),
            aggregate: serde_json::to_value(TestAggregate {
                something_happened: 3,
            })
            .unwrap(),
            current_sequence: 3,
            current_snapshot: 2,
        })));
        let store =
            PersistedEventStore::new(repo).with_storage_method(SourceOfTruth::AggregateStore);
        let snapshot_context = store.load_aggregate(TEST_AGGREGATE_ID).await.unwrap();
        assert_eq!(Some(2), snapshot_context.current_snapshot);
        assert_eq!(3, snapshot_context.current_sequence);
        assert_eq!(TEST_AGGREGATE_ID, snapshot_context.aggregate_id);
        assert_eq!(
            TestAggregate {
                something_happened: 3
            },
            snapshot_context.aggregate
        );
    }

    #[tokio::test]
    async fn load_aggregate_error() {
        let repo = MockRepo::with_snapshot(Err(PersistenceError::OptimisticLockError));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new(repo)
            .with_storage_method(SourceOfTruth::AggregateStore);
        let result = store.load_aggregate(TEST_AGGREGATE_ID).await;
        match result {
            Err(AggregateError::AggregateConflict) => {}
            _ => panic!("expected technical error"),
        }
    }

    #[tokio::test]
    async fn commit() {
        let repo = MockRepo::with_commit(Box::new(|events, snapshot_update| {
            assert_eq!(3, events.len());
            let event = events.get(2).unwrap();
            assert_eq!(TEST_AGGREGATE_ID, event.aggregate_id);
            assert_eq!(3, event.sequence);

            let snapshot_update = snapshot_update.unwrap();
            let aggregate_id = snapshot_update.0;
            let aggregate = snapshot_update.1;
            let snapshot_version = snapshot_update.2;
            assert_eq!(TEST_AGGREGATE_ID, aggregate_id.as_str());
            assert_eq!(1, snapshot_version);
            assert_eq!(
                json!(TestAggregate {
                    something_happened: 2
                }),
                aggregate
            );
        }));
        let store =
            PersistedEventStore::new(repo).with_storage_method(SourceOfTruth::AggregateStore);
        // let store = PersistedSnapshotStore::new(repo);
        let context = EventStoreAggregateContext {
            aggregate_id: TEST_AGGREGATE_ID.to_string(),
            aggregate: TestAggregate::default(),
            current_sequence: 0,
            current_snapshot: Some(0),
        };
        let event_envelopes = store
            .commit(
                vec![
                    TestEvents::Started,
                    TestEvents::SomethingWasDone,
                    TestEvents::SomethingWasDone,
                ],
                context,
                HashMap::default(),
            )
            .await
            .unwrap();
        assert_eq!(3, event_envelopes.len());
        let event = event_envelopes.get(0).unwrap();
        assert_eq!(TEST_AGGREGATE_ID, event.aggregate_id);
        assert_eq!(TestEvents::Started, event.payload);
        assert_eq!(
            TestEvents::SomethingWasDone,
            event_envelopes.get(2).unwrap().payload
        );
    }

    pub(crate) fn test_serialized_event(seq: usize, event: TestEvents) -> SerializedEvent {
        let event_type = event.event_type();
        let event_version = event.event_version();
        let payload = serde_json::to_value(&event).unwrap();
        SerializedEvent::new(
            TEST_AGGREGATE_ID.to_string(),
            seq,
            "TestAggregate".to_string(),
            event_type,
            event_version,
            payload,
            serde_json::to_value(HashMap::<String, String>::new()).unwrap(),
        )
    }
}
