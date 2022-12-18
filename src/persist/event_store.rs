use std::collections::HashMap;
use std::marker::PhantomData;

use async_trait::async_trait;
use serde_json::Value;

use crate::persist::serialized_event::{deserialize_events, serialize_events};
use crate::persist::{
    EventStoreAggregateContext, EventUpcaster, PersistedEventRepository, SerializedEvent,
};
use crate::{Aggregate, AggregateError, EventEnvelope, EventStore};

enum SourceOfTruth {
    EventStore,
    Snapshot(usize),
    AggregateStore,
}

impl SourceOfTruth {
    fn commit_snapshot_with_addl_events(
        &self,
        current_sequence: usize,
        num_events: usize,
    ) -> usize {
        match self {
            Self::EventStore => 0,
            Self::Snapshot(max_size) => {
                let next_snapshot_at = max_size - (current_sequence % max_size);
                if num_events < next_snapshot_at {
                    0
                } else {
                    let addl_events_after_next_snapshot = num_events - next_snapshot_at;
                    let addl_events_after_next_snapshot_to_apply = addl_events_after_next_snapshot
                        - (addl_events_after_next_snapshot % max_size);
                    next_snapshot_at + addl_events_after_next_snapshot_to_apply
                }
            }
            Self::AggregateStore => num_events,
        }
    }
}

#[test]
fn test_source_of_truth() {
    assert_eq!(
        0,
        SourceOfTruth::EventStore.commit_snapshot_with_addl_events(5, 3)
    );
    assert_eq!(
        3,
        SourceOfTruth::AggregateStore.commit_snapshot_with_addl_events(5, 3)
    );
    assert_eq!(
        0,
        SourceOfTruth::Snapshot(5).commit_snapshot_with_addl_events(5, 3)
    );
    assert_eq!(
        3,
        SourceOfTruth::Snapshot(4).commit_snapshot_with_addl_events(5, 3)
    );
    assert_eq!(
        3,
        SourceOfTruth::Snapshot(4).commit_snapshot_with_addl_events(5, 4)
    );
    assert_eq!(
        7,
        SourceOfTruth::Snapshot(4).commit_snapshot_with_addl_events(5, 8)
    );
}

/// Storage engine using a database backing.
/// This defaults to an event-sourced store (i.e., events are the single source of truth),
/// but can be configured to be aggregate-sourced or use snapshots when a large number of events
/// are associated with a single aggregate instance.
///
pub struct PersistedEventStore<R, A>
where
    R: PersistedEventRepository,
    A: Aggregate + Send + Sync,
{
    repo: R,
    storage: SourceOfTruth,
    event_upcasters: Option<Vec<Box<dyn EventUpcaster>>>,
    _phantom: PhantomData<A>,
}

impl<R, A> PersistedEventStore<R, A>
where
    R: PersistedEventRepository,
    A: Aggregate + Send + Sync,
{
    /// Creates a new `PersistedEventStore` from the provided event repository,
    /// using events as the single source of truth.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyService};
    /// # use cqrs_es::CqrsFramework;
    /// # use cqrs_es::persist::doc::{MyDatabaseConnection, MyEventRepository};
    /// # use cqrs_es::persist::PersistedEventStore;
    /// # fn config(my_db_connection: MyDatabaseConnection) {
    /// let repo = MyEventRepository::new(my_db_connection);
    /// let store = PersistedEventStore::<MyEventRepository,MyAggregate>::new_event_store(repo);
    /// let service = MyService;
    /// let cqrs = CqrsFramework::new(store, vec![], service);
    /// # }
    /// ```
    pub fn new_event_store(repo: R) -> Self {
        Self {
            repo,
            storage: SourceOfTruth::EventStore,
            event_upcasters: None,
            _phantom: PhantomData,
        }
    }

    /// Creates a new `PersistedEventStore` from the provided event repository,
    /// using the serialized aggregate as the source of truth.
    /// As with other `EventStore` implementations, committed events are stored but they are
    /// not used as the source of truth when loading an aggregate.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyService};
    /// # use cqrs_es::CqrsFramework;
    /// # use cqrs_es::persist::doc::{MyDatabaseConnection, MyEventRepository};
    /// # use cqrs_es::persist::PersistedEventStore;
    /// # fn config(my_db_connection: MyDatabaseConnection) {
    /// let repo = MyEventRepository::new(my_db_connection);
    /// let store = PersistedEventStore::<MyEventRepository,MyAggregate>::new_aggregate_store(repo);
    /// let cqrs = CqrsFramework::new(store, vec![], MyService);
    /// # }
    /// ```
    pub fn new_aggregate_store(repo: R) -> Self {
        Self {
            repo,
            storage: SourceOfTruth::AggregateStore,
            event_upcasters: None,
            _phantom: PhantomData,
        }
    }

    /// Creates a new `PersistedEventStore` from the provided event repository,
    /// using events and aggregate snapshots as the source of truth.
    ///
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyService};
    /// # use cqrs_es::CqrsFramework;
    /// # use cqrs_es::persist::doc::{MyDatabaseConnection, MyEventRepository};
    /// # use cqrs_es::persist::PersistedEventStore;
    /// # fn config(my_db_connection: MyDatabaseConnection) {
    /// let repo = MyEventRepository::new(my_db_connection);
    /// let store = PersistedEventStore::<MyEventRepository,MyAggregate>::new_snapshot_store(repo, 100);
    /// let cqrs = CqrsFramework::new(store, vec![], MyService);
    /// # }
    /// ```
    pub fn new_snapshot_store(repo: R, snapshot_size: usize) -> Self {
        Self {
            repo,
            storage: SourceOfTruth::Snapshot(snapshot_size),
            event_upcasters: None,
            _phantom: PhantomData,
        }
    }

    /// Configures the event store to use event upcasters when loading events.
    /// The EventUpcasters within the Vec should be placed in the
    /// order that they should be applied
    ///
    /// E.g., an upcaster for version 0.2.3 should be placed before an upcaster for version 0.2.4
    pub fn with_upcasters(self, event_upcasters: Vec<Box<dyn EventUpcaster>>) -> Self {
        Self {
            repo: self.repo,
            storage: self.storage,
            event_upcasters: Some(event_upcasters),
            _phantom: PhantomData::default(),
        }
    }
}

#[async_trait]
impl<R, A> EventStore<A> for PersistedEventStore<R, A>
where
    R: PersistedEventRepository,
    A: Aggregate + Send + Sync,
{
    type AC = EventStoreAggregateContext<A>;

    async fn load_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError<A::Error>> {
        let serialized_events = self.repo.get_events::<A>(aggregate_id).await?;
        Ok(deserialize_events(
            serialized_events,
            &self.event_upcasters,
        )?)
    }

    async fn load_aggregate(
        &self,
        aggregate_id: &str,
    ) -> Result<EventStoreAggregateContext<A>, AggregateError<A::Error>> {
        let mut context: EventStoreAggregateContext<A> =
            if let SourceOfTruth::EventStore = self.storage {
                EventStoreAggregateContext::context_for(aggregate_id, true)
            } else {
                let snapshot = self.repo.get_snapshot::<A>(aggregate_id).await?;
                match snapshot {
                    Some(snapshot) => snapshot.try_into()?,
                    None => EventStoreAggregateContext::context_for(aggregate_id, false),
                }
            };
        let events_to_apply = match self.storage {
            SourceOfTruth::EventStore => self.load_events(aggregate_id).await?,
            SourceOfTruth::Snapshot(_) => {
                let serialized_events = self
                    .repo
                    .get_last_events::<A>(aggregate_id, context.current_sequence)
                    .await?;
                deserialize_events(serialized_events, &self.event_upcasters)?
            }
            SourceOfTruth::AggregateStore => {
                vec![]
            }
        };
        for envelope in events_to_apply {
            context.current_sequence = envelope.sequence;
            let event = envelope.payload;
            context.aggregate.apply(event);
        }
        Ok(context)
    }

    async fn commit(
        &self,
        events: Vec<A::Event>,
        context: EventStoreAggregateContext<A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError<A::Error>> {
        let aggregate_id = context.aggregate_id.clone();
        let last_sequence = context.current_sequence;

        let commit_snapshot_to_event = self
            .storage
            .commit_snapshot_with_addl_events(context.current_sequence, events.len());
        let snapshot_update: Option<(Value, usize)> = if commit_snapshot_to_event == 0 {
            None
        } else {
            match self.storage {
                SourceOfTruth::EventStore => None,
                _ => Self::update_snapshot_with_events(&events, context, commit_snapshot_to_event)?,
            }
        };
        let wrapped_events = self.wrap_events(&aggregate_id, last_sequence, events, metadata);
        let serialized_events: Vec<SerializedEvent> = serialize_events(&wrapped_events)?;
        let snapshot_update = snapshot_update.map(|s| (aggregate_id, s.0, s.1));
        self.repo
            .persist::<A>(&serialized_events, snapshot_update)
            .await?;
        Ok(wrapped_events)
    }
}

impl<R, A> PersistedEventStore<R, A>
where
    A: Aggregate + Send + Sync,
    R: PersistedEventRepository,
{
    fn update_snapshot_with_events(
        events: &[<A as Aggregate>::Event],
        mut context: EventStoreAggregateContext<A>,
        commit_snapshot_to_event: usize,
    ) -> Result<Option<(Value, usize)>, AggregateError<A::Error>> {
        for (i, event) in events.iter().cloned().enumerate() {
            if i < commit_snapshot_to_event {
                context.aggregate.apply(event);
                context.current_sequence += 1;
            }
        }
        let next_snapshot = context.current_snapshot.map_or(1, |val| val + 1);
        let payload = serde_json::to_value(context.aggregate)?;
        Ok(Some((payload, next_snapshot)))
    }

    /// Method to wrap a set of events with the additional metadata needed for persistence and publishing
    fn wrap_events(
        &self,
        aggregate_id: &str,
        last_sequence: usize,
        resultant_events: Vec<A::Event>,
        base_metadata: HashMap<String, String>,
    ) -> Vec<EventEnvelope<A>> {
        let mut sequence = last_sequence;
        let mut wrapped_events: Vec<EventEnvelope<A>> = Vec::new();
        for payload in resultant_events {
            sequence += 1;
            let aggregate_id: String = aggregate_id.to_string();
            let sequence = sequence;
            let metadata = base_metadata.clone();
            wrapped_events.push(EventEnvelope {
                aggregate_id,
                sequence,
                payload,
                metadata,
            });
        }
        wrapped_events
    }
}

#[cfg(test)]
pub(crate) mod shared_test {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    use crate::persist::event_stream::ReplayStream;
    use crate::persist::{
        PersistedEventRepository, PersistenceError, SerializedEvent, SerializedSnapshot,
    };
    use crate::{Aggregate, DomainEvent};

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    pub(crate) enum TestEvents {
        Started,
        SomethingWasDone,
    }

    impl DomainEvent for TestEvents {
        fn event_type(&self) -> String {
            match self {
                Self::Started => "Started".to_string(),
                Self::SomethingWasDone => "SomethingWasDone".to_string(),
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

    #[derive(Debug, thiserror::Error)]
    #[error("{0}")]
    pub(crate) struct TestError(String);

    impl From<&str> for TestError {
        fn from(msg: &str) -> Self {
            Self(msg.to_string())
        }
    }

    #[derive(Clone)]
    pub struct TestService;

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    pub(crate) struct TestAggregate {
        pub(crate) something_happened: usize,
    }

    #[async_trait]
    impl Aggregate for TestAggregate {
        type Command = TestCommands;
        type Event = TestEvents;
        type Error = TestError;
        type Services = TestService;

        fn aggregate_type() -> String {
            "TestAggregate".to_string()
        }
        async fn handle(
            &self,
            command: Self::Command,
            _service: &Self::Services,
        ) -> Result<Vec<Self::Event>, Self::Error> {
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

    pub(crate) struct MockEventIterator;
    impl Iterator for MockEventIterator {
        type Item = Result<SerializedEvent, PersistenceError>;

        fn next(&mut self) -> Option<Self::Item> {
            todo!()
        }
    }

    pub(crate) struct MockRepo {
        events_result: Mutex<Option<Result<Vec<SerializedEvent>, PersistenceError>>>,
        last_events_result: Mutex<Option<Result<Vec<SerializedEvent>, PersistenceError>>>,
        snapshot_result: Mutex<Option<Result<Option<SerializedSnapshot>, PersistenceError>>>,
        #[allow(clippy::type_complexity)]
        persist_check: Mutex<
            Option<Box<dyn FnOnce(&[SerializedEvent], Option<(String, Value, usize)>) + Send>>,
        >,
    }

    impl MockRepo {
        pub(crate) fn with_events(result: Result<Vec<SerializedEvent>, PersistenceError>) -> Self {
            Self {
                events_result: Mutex::new(Some(result)),
                last_events_result: Mutex::new(None),
                snapshot_result: Mutex::new(None),
                persist_check: Mutex::new(None),
            }
        }
        pub(crate) fn with_last_events(
            last_events: Result<Vec<SerializedEvent>, PersistenceError>,
            snapshot: Result<Option<SerializedSnapshot>, PersistenceError>,
        ) -> Self {
            Self {
                events_result: Mutex::new(None),
                last_events_result: Mutex::new(Some(last_events)),
                snapshot_result: Mutex::new(Some(snapshot)),
                persist_check: Mutex::new(None),
            }
        }
        pub(crate) fn with_snapshot(
            result: Result<Option<SerializedSnapshot>, PersistenceError>,
        ) -> Self {
            Self {
                events_result: Mutex::new(None),
                last_events_result: Mutex::new(None),
                snapshot_result: Mutex::new(Some(result)),
                persist_check: Mutex::new(None),
            }
        }
        #[allow(clippy::type_complexity)]
        pub(crate) fn with_commit(
            test_function: Box<
                dyn FnOnce(&[SerializedEvent], Option<(String, Value, usize)>) + Send,
            >,
        ) -> Self {
            Self {
                events_result: Mutex::new(None),
                last_events_result: Mutex::new(None),
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
            self.last_events_result.lock().unwrap().take().unwrap()
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

        async fn stream_events<A: Aggregate>(
            &self,
            _aggregate_id: &str,
        ) -> Result<ReplayStream, PersistenceError> {
            self.stream_all_events::<A>().await
        }

        async fn stream_all_events<A: Aggregate>(&self) -> Result<ReplayStream, PersistenceError> {
            let result = self.events_result.lock().unwrap().take().unwrap();
            match result {
                Ok(events) => {
                    let (mut feed, stream) = ReplayStream::new(events.len());
                    for event in events {
                        feed.push(Ok(event)).await?;
                    }
                    Ok(stream)
                }
                Err(err) => Err(err),
            }
        }
    }

    pub(crate) const TEST_AGGREGATE_ID: &str = "test-aggregate-C";
    pub(crate) const EVENT_VERSION: &str = "1.0";

    pub(crate) fn test_serialized_event(seq: usize, event: TestEvents) -> SerializedEvent {
        let event_type = event.event_type();
        let event_version = event.event_version();
        let payload = serde_json::to_value(event).unwrap();
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

#[cfg(test)]
mod event_store_test {
    use std::collections::HashMap;

    use crate::persist::event_store::shared_test::{
        test_serialized_event, MockRepo, TestAggregate, TestEvents, EVENT_VERSION,
        TEST_AGGREGATE_ID,
    };
    use crate::persist::{EventStoreAggregateContext, PersistedEventStore, PersistenceError};
    use crate::{AggregateError, DomainEvent, EventStore};

    #[tokio::test]
    async fn load() {
        let repo = MockRepo::with_events(Ok(vec![test_serialized_event(
            1,
            TestEvents::SomethingWasDone,
        )]));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_event_store(repo);
        let events = store.load_events(TEST_AGGREGATE_ID).await.unwrap();
        let event = events.get(0).unwrap();
        assert_eq!(1, event.sequence);
        assert_eq!("SomethingWasDone", event.payload.event_type());
        assert_eq!(EVENT_VERSION, event.payload.event_version());
    }

    #[tokio::test]
    async fn load_error() {
        let repo = MockRepo::with_events(Err(PersistenceError::OptimisticLockError));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_event_store(repo);
        let result = store.load_events(TEST_AGGREGATE_ID).await;
        match result {
            Err(AggregateError::AggregateConflict) => {}
            _ => panic!("expected technical error"),
        }
    }

    #[tokio::test]
    async fn load_aggregate_new() {
        let repo = MockRepo::with_events(Ok(vec![]));
        let store = PersistedEventStore::new_event_store(repo);
        let agg_context = store.load_aggregate(TEST_AGGREGATE_ID).await.unwrap();
        assert_eq!(None, agg_context.current_snapshot);
        assert_eq!(0, agg_context.current_sequence);
        assert_eq!(TEST_AGGREGATE_ID, agg_context.aggregate_id);
        assert_eq!(TestAggregate::default(), agg_context.aggregate);
    }

    #[tokio::test]
    async fn load_aggregate_existing() {
        let repo = MockRepo::with_events(Ok(vec![
            test_serialized_event(1, TestEvents::Started),
            test_serialized_event(2, TestEvents::SomethingWasDone),
        ]));
        let store = PersistedEventStore::new_event_store(repo);
        let snapshot_context = store.load_aggregate(TEST_AGGREGATE_ID).await.unwrap();
        assert_eq!(None, snapshot_context.current_snapshot);
        assert_eq!(2, snapshot_context.current_sequence);
        assert_eq!(TEST_AGGREGATE_ID, snapshot_context.aggregate_id);
        assert_eq!(
            TestAggregate {
                something_happened: 1
            },
            snapshot_context.aggregate
        );
    }

    #[tokio::test]
    async fn load_aggregate_error() {
        let repo = MockRepo::with_events(Err(PersistenceError::OptimisticLockError));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_event_store(repo);
        let result = store.load_aggregate(TEST_AGGREGATE_ID).await;
        match result {
            Err(AggregateError::AggregateConflict) => {}
            _ => panic!("expected aggregate conflict"),
        }
    }

    #[tokio::test]
    async fn commit() {
        let repo = MockRepo::with_commit(Box::new(|events, snapshot_update| {
            assert_eq!(3, events.len());
            let event = events.get(2).unwrap();
            assert_eq!(TEST_AGGREGATE_ID, event.aggregate_id);
            assert_eq!(3, event.sequence);

            assert!(snapshot_update.is_none());
        }));
        let store = PersistedEventStore::new_event_store(repo);
        let context = EventStoreAggregateContext {
            aggregate_id: TEST_AGGREGATE_ID.to_string(),
            aggregate: TestAggregate::default(),
            current_sequence: 0,
            current_snapshot: None,
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
}

#[cfg(test)]
pub(crate) mod snapshotted_store_test {
    use std::collections::HashMap;

    use serde_json::json;

    use crate::persist::event_store::shared_test::{
        test_serialized_event, MockRepo, TestAggregate, TestEvents, EVENT_VERSION,
        TEST_AGGREGATE_ID,
    };
    use crate::persist::{
        EventStoreAggregateContext, PersistedEventStore, PersistenceError, SerializedSnapshot,
    };
    use crate::{AggregateError, DomainEvent, EventStore};

    #[tokio::test]
    async fn load() {
        let repo = MockRepo::with_events(Ok(vec![test_serialized_event(
            1,
            TestEvents::SomethingWasDone,
        )]));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_snapshot_store(repo, 2);
        let events = store.load_events(TEST_AGGREGATE_ID).await.unwrap();
        let event = events.get(0).unwrap();
        assert_eq!(1, event.sequence);
        assert_eq!("SomethingWasDone", event.payload.event_type());
        assert_eq!(EVENT_VERSION, event.payload.event_version());
    }

    #[tokio::test]
    async fn load_error() {
        let repo = MockRepo::with_events(Err(PersistenceError::OptimisticLockError));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_snapshot_store(repo, 2);
        let result = store.load_events(TEST_AGGREGATE_ID).await.unwrap_err();
        match result {
            AggregateError::AggregateConflict => {}
            _ => panic!("expected technical error"),
        }
    }

    #[tokio::test]
    async fn load_aggregate_new() {
        let repo = MockRepo::with_last_events(Ok(vec![]), Ok(None));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_snapshot_store(repo, 2);
        let snapshot_context = store.load_aggregate(TEST_AGGREGATE_ID).await.unwrap();
        assert_eq!(None, snapshot_context.current_snapshot);
        assert_eq!(0, snapshot_context.current_sequence);
        assert_eq!(TEST_AGGREGATE_ID, snapshot_context.aggregate_id);
        assert_eq!(TestAggregate::default(), snapshot_context.aggregate);
    }

    #[tokio::test]
    async fn load_aggregate_existing() {
        let repo = MockRepo::with_last_events(
            Ok(vec![test_serialized_event(4, TestEvents::SomethingWasDone)]),
            Ok(Some(SerializedSnapshot {
                aggregate_id: TEST_AGGREGATE_ID.to_string(),
                aggregate: serde_json::to_value(TestAggregate {
                    something_happened: 3,
                })
                .unwrap(),
                current_sequence: 3,
                current_snapshot: 2,
            })),
        );
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_snapshot_store(repo, 2);
        let snapshot_context = store.load_aggregate(TEST_AGGREGATE_ID).await.unwrap();
        assert_eq!(Some(2), snapshot_context.current_snapshot);
        assert_eq!(4, snapshot_context.current_sequence);
        assert_eq!(TEST_AGGREGATE_ID, snapshot_context.aggregate_id);
        assert_eq!(
            TestAggregate {
                something_happened: 4
            },
            snapshot_context.aggregate
        );
    }

    #[tokio::test]
    async fn load_aggregate_error() {
        let repo = MockRepo::with_snapshot(Err(PersistenceError::OptimisticLockError));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_snapshot_store(repo, 2);
        let result = store.load_aggregate(TEST_AGGREGATE_ID).await;
        match result {
            Err(AggregateError::AggregateConflict) => {}
            _ => panic!("expected technical error"),
        }
    }

    #[tokio::test]
    async fn commit_one_event() {
        let repo = MockRepo::with_commit(Box::new(|events, snapshot_update| {
            assert_eq!(1, events.len());
            let event = events.get(0).unwrap();
            assert_eq!(TEST_AGGREGATE_ID, event.aggregate_id);
            assert_eq!(1, event.sequence);

            assert_eq!(None, snapshot_update);
        }));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_snapshot_store(repo, 2);
        let context = EventStoreAggregateContext {
            aggregate_id: TEST_AGGREGATE_ID.to_string(),
            aggregate: TestAggregate::default(),
            current_sequence: 0,
            current_snapshot: Some(0),
        };
        let event_envelopes = store
            .commit(vec![TestEvents::Started], context, HashMap::default())
            .await
            .unwrap();
        assert_eq!(1, event_envelopes.len());
        let event = event_envelopes.get(0).unwrap();
        assert_eq!(TEST_AGGREGATE_ID, event.aggregate_id);
        assert_eq!(TestEvents::Started, event.payload);
    }

    #[tokio::test]
    async fn commit_one_event_with_previous() {
        let repo = MockRepo::with_commit(Box::new(|events, snapshot_update| {
            assert_eq!(1, events.len());
            let event = events.get(0).unwrap();
            assert_eq!(TEST_AGGREGATE_ID, event.aggregate_id);
            assert_eq!(3, event.sequence);

            assert_eq!(None, snapshot_update);
        }));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_snapshot_store(repo, 2);
        let context = EventStoreAggregateContext {
            aggregate_id: TEST_AGGREGATE_ID.to_string(),
            aggregate: TestAggregate::default(),
            current_sequence: 2,
            current_snapshot: Some(1),
        };
        let event_envelopes = store
            .commit(
                vec![TestEvents::SomethingWasDone],
                context,
                HashMap::default(),
            )
            .await
            .unwrap();
        assert_eq!(1, event_envelopes.len());
        let event = event_envelopes.get(0).unwrap();
        assert_eq!(TEST_AGGREGATE_ID, event.aggregate_id);
        assert_eq!(TestEvents::SomethingWasDone, event.payload);
    }

    #[tokio::test]
    async fn commit_three_events() {
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
                    something_happened: 1
                }),
                aggregate
            );
        }));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_snapshot_store(repo, 2);
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

    #[tokio::test]
    async fn commit_two_with_existing() {
        let repo = MockRepo::with_commit(Box::new(|events, snapshot_update| {
            assert_eq!(2, events.len());
            let event = events.get(1).unwrap();
            assert_eq!(TEST_AGGREGATE_ID, event.aggregate_id);
            assert_eq!(3, event.sequence);

            let snapshot_update = snapshot_update.unwrap();
            let aggregate_id = snapshot_update.0;
            let aggregate = snapshot_update.1;
            let snapshot_version = snapshot_update.2;
            assert_eq!(TEST_AGGREGATE_ID, aggregate_id.as_str());
            assert_eq!(2, snapshot_version);
            assert_eq!(
                json!(TestAggregate {
                    something_happened: 1
                }),
                aggregate
            );
        }));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_snapshot_store(repo, 2);
        let context = EventStoreAggregateContext {
            aggregate_id: TEST_AGGREGATE_ID.to_string(),
            aggregate: TestAggregate::default(),
            current_sequence: 1,
            current_snapshot: Some(1),
        };
        let event_envelopes = store
            .commit(
                vec![TestEvents::SomethingWasDone, TestEvents::SomethingWasDone],
                context,
                HashMap::default(),
            )
            .await
            .unwrap();
        assert_eq!(2, event_envelopes.len());
        let first_event = event_envelopes.get(0).unwrap();
        assert_eq!(TEST_AGGREGATE_ID, first_event.aggregate_id);
        assert_eq!(TestEvents::SomethingWasDone, first_event.payload);
        let last_event = event_envelopes.get(1).unwrap();
        assert_eq!(TEST_AGGREGATE_ID, last_event.aggregate_id);
        assert_eq!(TestEvents::SomethingWasDone, last_event.payload);
    }

    #[tokio::test]
    async fn commit_five() {
        let repo = MockRepo::with_commit(Box::new(|events, snapshot_update| {
            assert_eq!(5, events.len());
            let event = events.get(4).unwrap();
            assert_eq!(TEST_AGGREGATE_ID, event.aggregate_id);
            assert_eq!(5, event.sequence);

            let snapshot_update = snapshot_update.unwrap();
            let aggregate_id = snapshot_update.0;
            let aggregate = snapshot_update.1;
            let snapshot_version = snapshot_update.2;
            assert_eq!(TEST_AGGREGATE_ID, aggregate_id.as_str());
            assert_eq!(1, snapshot_version);
            assert_eq!(
                json!(TestAggregate {
                    something_happened: 3
                }),
                aggregate
            );
        }));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_snapshot_store(repo, 2);
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
                    TestEvents::SomethingWasDone,
                    TestEvents::SomethingWasDone,
                ],
                context,
                HashMap::default(),
            )
            .await
            .unwrap();
        assert_eq!(5, event_envelopes.len());
        let first_event = event_envelopes.get(0).unwrap();
        assert_eq!(TEST_AGGREGATE_ID, first_event.aggregate_id);
        assert_eq!(TestEvents::Started, first_event.payload);
        let last_event = event_envelopes.get(4).unwrap();
        assert_eq!(TEST_AGGREGATE_ID, last_event.aggregate_id);
        assert_eq!(TestEvents::SomethingWasDone, last_event.payload);
    }
}

#[cfg(test)]
pub(crate) mod aggregate_store_test {
    use std::collections::HashMap;

    use serde_json::json;

    use crate::persist::event_store::shared_test::{
        test_serialized_event, MockRepo, TestAggregate, TestEvents, EVENT_VERSION,
        TEST_AGGREGATE_ID,
    };
    use crate::persist::{
        EventStoreAggregateContext, PersistedEventStore, PersistenceError, SerializedSnapshot,
    };
    use crate::{AggregateError, DomainEvent, EventStore};

    #[tokio::test]
    async fn load() {
        let repo = MockRepo::with_events(Ok(vec![test_serialized_event(
            1,
            TestEvents::SomethingWasDone,
        )]));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_aggregate_store(repo);
        let events = store.load_events(TEST_AGGREGATE_ID).await.unwrap();
        let event = events.get(0).unwrap();
        assert_eq!(1, event.sequence);
        assert_eq!("SomethingWasDone", event.payload.event_type());
        assert_eq!(EVENT_VERSION, event.payload.event_version());
    }

    #[tokio::test]
    async fn load_error() {
        let repo = MockRepo::with_events(Err(PersistenceError::OptimisticLockError));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_aggregate_store(repo);
        let result = store.load_events(TEST_AGGREGATE_ID).await.unwrap_err();
        match result {
            AggregateError::AggregateConflict => {}
            _ => panic!("expected technical error"),
        }
    }

    #[tokio::test]
    async fn load_aggregate_new() {
        let repo = MockRepo::with_snapshot(Ok(None));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_aggregate_store(repo);
        let snapshot_context = store.load_aggregate(TEST_AGGREGATE_ID).await.unwrap();
        assert_eq!(None, snapshot_context.current_snapshot);
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
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_aggregate_store(repo);
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
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_aggregate_store(repo);
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
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new_aggregate_store(repo);
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
}
