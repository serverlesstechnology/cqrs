use std::collections::HashMap;
use std::marker::PhantomData;

use crate::persist::{
    PersistedEventRepository, PersistedSnapshotEventRepository, SnapshotStoreAggregateContext,
};
use crate::{Aggregate, AggregateError, EventEnvelope, EventStore};
use async_trait::async_trait;

/// Storage engine using a database backing.
/// This is an snapshot-sourced `EventStore`, meaning it uses the serialized aggregate as the
/// primary source of truth for the state of the aggregate.
///
/// The individual events are also persisted but are used only for updating queries.
///
/// For a event-sourced `EventStore` see [`PersistedEventStore`](struct.PersistedEventStore.html).
///
pub struct PersistedSnapshotStore<R, SR, A>
where
    R: PersistedEventRepository<A>,
    SR: PersistedSnapshotEventRepository<A>,
    A: Aggregate + Send + Sync,
{
    event_repo: R,
    snapshot_repo: SR,
    _phantom: PhantomData<A>,
}

impl<R, SR, A> PersistedSnapshotStore<R, SR, A>
where
    R: PersistedEventRepository<A>,
    SR: PersistedSnapshotEventRepository<A>,
    A: Aggregate + Send + Sync,
{
    /// Creates a new `PostgresSnapshotStore` from the provided database connection pool,
    /// an `EventStore` used for configuring a new cqrs framework.
    ///
    /// ```ignore
    /// # use postgres_es::PostgresSnapshotStore;
    /// # use cqrs_es::CqrsFramework;
    /// let store = PostgresSnapshotStore::<MyAggregate>::new(pool);
    /// let cqrs = CqrsFramework::new(store, vec![]);
    /// ```
    pub fn new(snapshot_repo: SR, event_repo: R) -> Self {
        PersistedSnapshotStore {
            event_repo,
            snapshot_repo,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<R, SR, A> EventStore<A> for PersistedSnapshotStore<R, SR, A>
where
    R: PersistedEventRepository<A>,
    SR: PersistedSnapshotEventRepository<A>,
    A: Aggregate + Send + Sync,
{
    type AC = SnapshotStoreAggregateContext<A>;

    async fn load(&self, aggregate_id: &str) -> Vec<EventEnvelope<A>> {
        match self.event_repo.get_events(aggregate_id).await {
            Ok(val) => val,
            Err(_err) => {
                // TODO: improved error handling
                Default::default()
            }
        }
    }
    async fn load_aggregate(&self, aggregate_id: &str) -> SnapshotStoreAggregateContext<A> {
        match self.snapshot_repo.get_snapshot(aggregate_id).await {
            Ok(snapshot) => match snapshot {
                Some(snapshot) => {
                    let _tmp = serde_json::to_string(&snapshot.aggregate).unwrap();
                    snapshot
                }
                None => SnapshotStoreAggregateContext {
                    aggregate_id: aggregate_id.to_string(),
                    aggregate: Default::default(),
                    current_sequence: 0,
                    current_snapshot: 0,
                },
            },
            Err(e) => {
                panic!("{}", e);
            }
        }
    }

    async fn commit(
        &self,
        events: Vec<A::Event>,
        mut context: SnapshotStoreAggregateContext<A>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError> {
        for event in events.clone() {
            context.aggregate.apply(event);
        }
        let aggregate_id = context.aggregate_id.clone();
        let next_snapshot = context.current_snapshot + 1;
        let wrapped_events =
            self.wrap_events(&aggregate_id, context.current_sequence, events, metadata);
        self.snapshot_repo
            .persist(
                context.aggregate,
                aggregate_id,
                next_snapshot,
                &wrapped_events,
            )
            .await?;
        Ok(wrapped_events)
    }
}
