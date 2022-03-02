use std::collections::HashMap;
use std::marker::PhantomData;

use async_trait::async_trait;

use crate::persist::serialized_event::{deserialize_events, serialize_events};
use crate::persist::{
    EventStoreAggregateContext, EventUpcaster, PersistedEventRepository, SerializedEvent,
};
use crate::{Aggregate, AggregateError, EventEnvelope, EventStore};

/// Denotes the source of truth used by the framework when loading an aggregate for processing
/// a command.
pub enum SourceOfTruth {
    /// All of the preceding events will be loaded, using these solely to prepare the aggregate state.
    EventStore,
    /// Will attempt to use events as the single source of truth up to a certain threshold of
    /// events, after which it will store snapshots and load only the most recent events.
    Snapshot(usize),
    /// The aggregate will be deserialized directly from the database.
    /// This is the simplest storage form to manage (and likely the most familiar) but offers the
    /// least flexibility when business rules change.
    AggregateStore,
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
    /// Creates a new `PostgresStore` from the provided event repository,
    /// an `EventStore` used for configuring a new cqrs framework.
    ///
    /// ```
    /// # use cqrs_es::doc::MyAggregate;
    /// # use cqrs_es::CqrsFramework;
    /// # use cqrs_es::persist::doc::{MyDatabaseConnection, MyEventRepository};
    /// # use cqrs_es::persist::PersistedEventStore;
    /// # fn config(my_db_connection: MyDatabaseConnection) {
    /// let repo = MyEventRepository::new(my_db_connection);
    /// let store = PersistedEventStore::<MyEventRepository,MyAggregate>::new(repo);
    /// let cqrs = CqrsFramework::new(store, vec![]);
    /// # }
    /// ```
    pub fn new(repo: R) -> Self {
        PersistedEventStore {
            repo,
            storage: SourceOfTruth::EventStore,
            event_upcasters: None,
            _phantom: PhantomData,
        }
    }

    /// Configures the source of truth used by the EventStore. All committed events will be stored
    /// but not necessarily loaded when processing inbound commands.
    ///
    /// ```rust
    /// # use cqrs_es::doc::{MyAggregate, MyRepository};
    /// # use cqrs_es::persist::{PersistedEventStore,SourceOfTruth};
    /// # async fn config(repo: MyRepository) {
    ///     let event_store = PersistedEventStore::<MyRepository, MyAggregate>::new(repo)
    ///         .with_storage_method(SourceOfTruth::AggregateStore);
    /// # }
    /// ```
    ///
    /// Changing storage methods of in-use databases is not supported out of the box.
    pub fn with_storage_method(self, storage: SourceOfTruth) -> Self {
        Self {
            repo: self.repo,
            storage,
            event_upcasters: self.event_upcasters,
            _phantom: Default::default(),
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
            _phantom: Default::default(),
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
        PersistedEventStore::load_from_repo(&self.repo, aggregate_id, &self.event_upcasters).await
    }

    async fn load_aggregate(
        &self,
        aggregate_id: &str,
    ) -> Result<EventStoreAggregateContext<A>, AggregateError<A::Error>> {
        match self.storage {
            SourceOfTruth::EventStore => {
                let committed_events = self.load_events(aggregate_id).await?;
                let mut aggregate = A::default();
                let mut current_sequence = 0;
                for envelope in committed_events {
                    current_sequence = envelope.sequence;
                    let event = envelope.payload;
                    aggregate.apply(event);
                }
                Ok(EventStoreAggregateContext {
                    aggregate_id: aggregate_id.to_string(),
                    aggregate,
                    current_sequence,
                    current_snapshot: None,
                })
            }
            SourceOfTruth::Snapshot(_) => {
                todo!()
            }
            SourceOfTruth::AggregateStore => {
                let snapshot = self.repo.get_snapshot::<A>(aggregate_id).await?;
                match snapshot {
                    Some(snapshot) => Ok(snapshot.try_into()?),
                    None => Ok(EventStoreAggregateContext {
                        aggregate_id: aggregate_id.to_string(),
                        aggregate: Default::default(),
                        current_sequence: 0,
                        current_snapshot: Some(0),
                    }),
                }
            }
        }
    }

    async fn commit(
        &self,
        events: Vec<A::Event>,
        mut context: EventStoreAggregateContext<A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError<A::Error>> {
        match self.storage {
            SourceOfTruth::EventStore => {
                let aggregate_id = context.aggregate_id.as_str();
                let current_sequence = context.current_sequence;
                let wrapped_events =
                    self.wrap_events(aggregate_id, current_sequence, events, metadata);
                let serialized_events: Vec<SerializedEvent> = serialize_events(&wrapped_events)?;
                self.repo.persist::<A>(&serialized_events, None).await?;
                Ok(wrapped_events)
            }
            SourceOfTruth::Snapshot(_) => {
                todo!()
            }
            SourceOfTruth::AggregateStore => {
                for event in events.clone() {
                    context.aggregate.apply(event);
                }
                let aggregate_id = context.aggregate_id.clone();
                let next_snapshot = match context.current_snapshot {
                    Some(val) => val + 1,
                    None => 1,
                };
                let seq = context.current_sequence;
                let wrapped_events = self.wrap_events(&aggregate_id, seq, events, metadata);
                let serialized_events = serialize_events(&wrapped_events)?;
                let payload = serde_json::to_value(context.aggregate)?;
                let snapshot_update = Some((aggregate_id, payload, next_snapshot));
                self.repo
                    .persist::<A>(&serialized_events, snapshot_update)
                    .await?;
                Ok(wrapped_events)
            }
        }
    }
}

impl<R, A> PersistedEventStore<R, A>
where
    R: PersistedEventRepository,
    A: Aggregate + Send + Sync,
{
    pub(crate) async fn load_from_repo(
        repo: &R,
        aggregate_id: &str,
        upcasters: &Option<Vec<Box<dyn EventUpcaster>>>,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError<A::Error>> {
        let serialized_events = repo.get_events::<A>(aggregate_id).await?;
        Ok(deserialize_events(serialized_events, upcasters)?)
    }
    /// Method to wrap a set of events with the additional metadata needed for persistence and publishing
    fn wrap_events(
        &self,
        aggregate_id: &str,
        current_sequence: usize,
        resultant_events: Vec<A::Event>,
        base_metadata: HashMap<String, String>,
    ) -> Vec<EventEnvelope<A>> {
        let mut sequence = current_sequence;
        let mut wrapped_events: Vec<EventEnvelope<A>> = Vec::new();
        for payload in resultant_events {
            sequence += 1;
            let aggregate_type = A::aggregate_type().to_string();
            let aggregate_id: String = aggregate_id.to_string();
            let sequence = sequence;
            let metadata = base_metadata.clone();
            wrapped_events.push(EventEnvelope::new_with_metadata(
                aggregate_id,
                sequence,
                aggregate_type,
                payload,
                metadata,
            ));
        }
        wrapped_events
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::persist::snapshot_store::test::{
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
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new(repo);
        let events = store.load_events(TEST_AGGREGATE_ID).await.unwrap();
        let event = events.get(0).unwrap();
        assert_eq!(1, event.sequence);
        assert_eq!("SomethingWasDone", event.payload.event_type());
        assert_eq!(EVENT_VERSION, event.payload.event_version());
    }

    #[tokio::test]
    async fn load_error() {
        let repo = MockRepo::with_events(Err(PersistenceError::OptimisticLockError));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new(repo);
        let result = store.load_events(TEST_AGGREGATE_ID).await;
        match result {
            Err(AggregateError::AggregateConflict) => {}
            _ => panic!("expected technical error"),
        }
    }

    #[tokio::test]
    async fn load_aggregate_new() {
        let repo = MockRepo::with_events(Ok(vec![]));
        let store = PersistedEventStore::new(repo);
        let agg_context = store.load_aggregate(TEST_AGGREGATE_ID).await.unwrap();
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
        let store = PersistedEventStore::new(repo);
        let snapshot_context = store.load_aggregate(TEST_AGGREGATE_ID).await.unwrap();
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
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new(repo);
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
        let store = PersistedEventStore::new(repo);
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
