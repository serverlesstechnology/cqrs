use std::collections::HashMap;
use std::marker::PhantomData;

use async_trait::async_trait;

use crate::persist::serialized_event::{deserialize_events, serialize_events};
use crate::persist::{
    EventStoreAggregateContext, EventUpcaster, PersistedEventRepository, SerializedEvent,
};
use crate::{Aggregate, AggregateError, EventEnvelope, EventStore};

/// Storage engine using a database backing.
/// This is an event-sourced `EventStore`, meaning it uses events as the
/// primary source of truth for the state of the aggregate.
///
/// For a snapshot-based `EventStore`
/// see [`PersistedSnapshotStore`](struct.PersistedSnapshotStore.html).
///
pub struct PersistedEventStore<R, A>
where
    R: PersistedEventRepository,
    A: Aggregate + Send + Sync,
{
    repo: R,
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
            event_upcasters: None,
            _phantom: PhantomData,
        }
    }

    /// Adds configures the store to use event upcasters on load. These should be placed in the
    /// order that they are applied.
    ///
    /// E.g., an upcaster for version 0.2.3 should be placed before an upcaster for version 0.2.4
    pub fn with_upcasters(self, event_upcasters: Vec<Box<dyn EventUpcaster>>) -> Self {
        Self {
            repo: self.repo,
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

    async fn load(&self, aggregate_id: &str) -> Vec<EventEnvelope<A>> {
        PersistedEventStore::load_from_repo(&self.repo, aggregate_id, &self.event_upcasters).await
    }

    async fn load_aggregate(&self, aggregate_id: &str) -> EventStoreAggregateContext<A> {
        let committed_events = self.load(aggregate_id).await;
        let mut aggregate = A::default();
        let mut current_sequence = 0;
        for envelope in committed_events {
            current_sequence = envelope.sequence;
            let event = envelope.payload;
            aggregate.apply(event);
        }
        EventStoreAggregateContext {
            aggregate_id: aggregate_id.to_string(),
            aggregate,
            current_sequence,
        }
    }

    async fn commit(
        &self,
        events: Vec<A::Event>,
        context: EventStoreAggregateContext<A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError<A::Error>> {
        let aggregate_id = context.aggregate_id.as_str();
        let current_sequence = context.current_sequence;
        let wrapped_events = self.wrap_events(aggregate_id, current_sequence, events, metadata);
        let serialized_events: Vec<SerializedEvent> = serialize_events(&wrapped_events)?;
        self.repo.persist::<A>(&serialized_events, None).await?;
        Ok(wrapped_events)
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
    ) -> Vec<EventEnvelope<A>> {
        match repo.get_events::<A>(aggregate_id).await {
            Ok(serialized_events) => {
                match deserialize_events(serialized_events, upcasters) {
                    Ok(events) => events,
                    Err(_err) => {
                        // TODO: improved error handling (planned for v0.3.0)
                        Default::default()
                    }
                }
            }
            Err(_err) => {
                // TODO: improved error handling (planned for v0.3.0)
                Default::default()
            }
        }
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
    use crate::EventStore;

    #[tokio::test]
    async fn load() {
        let repo = MockRepo::with_events(Ok(vec![test_serialized_event(
            1,
            TestEvents::SomethingWasDone,
        )]));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new(repo);
        let events = store.load(TEST_AGGREGATE_ID).await;
        let event = events.get(0).unwrap();
        assert_eq!(1, event.sequence);
        assert_eq!("SomethingWasDone", event.event_type);
        assert_eq!(EVENT_VERSION, event.event_version);
    }

    #[tokio::test]
    async fn load_error() {
        let repo = MockRepo::with_events(Err(PersistenceError::OptimisticLockError));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new(repo);
        let events = store.load(TEST_AGGREGATE_ID).await;
        assert_eq!(0, events.len())
    }

    #[tokio::test]
    async fn load_aggregate_new() {
        let repo = MockRepo::with_events(Ok(vec![]));
        let store = PersistedEventStore::new(repo);
        let agg_context = store.load_aggregate(TEST_AGGREGATE_ID).await;
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
        let snapshot_context = store.load_aggregate(TEST_AGGREGATE_ID).await;
        assert_eq!(2, snapshot_context.current_sequence);
        assert_eq!(TEST_AGGREGATE_ID, snapshot_context.aggregate_id);
        assert_eq!(
            TestAggregate {
                something_happened: 1
            },
            snapshot_context.aggregate
        );
    }

    // TODO: better error handling needed, this panic could cause problems with non-severless systems
    #[tokio::test]
    #[should_panic]
    async fn load_aggregate_error() {
        let repo = MockRepo::with_snapshot(Err(PersistenceError::OptimisticLockError));
        let store = PersistedEventStore::<MockRepo, TestAggregate>::new(repo);
        store.load_aggregate(TEST_AGGREGATE_ID).await;
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
