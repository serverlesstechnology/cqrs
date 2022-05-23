use crate::persist::PersistedEventRepository;
use crate::persist::PersistedEventStore;
use crate::{Aggregate, AggregateError, EventStore, Query};

pub struct QueryReplay<R, Q, A>
where
    R: PersistedEventRepository,
    Q: Query<A>,
    A: Aggregate,
{
    event_store: PersistedEventStore<R, A>,
    query: Q,
}

impl<R, Q, A> QueryReplay<R, Q, A>
where
    R: PersistedEventRepository,
    Q: Query<A>,
    A: Aggregate,
{
    pub fn new(event_store: PersistedEventStore<R, A>, query: Q) -> Self {
        Self { event_store, query }
    }

    pub async fn replay(&self, aggregate_id: &str) -> Result<(), AggregateError<A::Error>> {
        let events = self.event_store.load_events(aggregate_id).await?;
        self.query.dispatch(aggregate_id, &events).await;
        Ok(())
    }

}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use crate::doc::{MyAggregate, MyEvents};
    use crate::persist::event_store::shared_test::MockRepo;
    use crate::persist::replay::QueryReplay;
    use crate::persist::{PersistedEventStore, SerializedEvent};
    use crate::{EventEnvelope, Query};

    #[derive(Debug)]
    struct MockQuery {
        events: Arc<Mutex<Vec<EventEnvelope<MyAggregate>>>>,
    }

    impl MockQuery {
        fn new() -> (Self, Arc<Mutex<Vec<EventEnvelope<MyAggregate>>>>) {
            let events: Arc<Mutex<Vec<EventEnvelope<MyAggregate>>>> = Default::default();
            let query = Self {
                events: events.clone(),
            };
            (query, events)
        }
    }

    #[async_trait]
    impl Query<MyAggregate> for MockQuery {
        async fn dispatch(&self, _aggregate_id: &str, events: &[EventEnvelope<MyAggregate>]) {
            let mut event_list = self.events.lock().unwrap();
            for event in events {
                event_list.push(event.to_owned());
            }
        }
    }

    const AGGREGATE_ID: &str = "test_aggregate";

    #[tokio::test]
    async fn query_replay() {
        let expected_events = vec![EventEnvelope {
            aggregate_id: AGGREGATE_ID.to_string(),
            sequence: 1,
            payload: MyEvents::SomethingWasDone,
            metadata: Default::default(),
        }];
        let ser_events = expected_events
            .iter()
            .map(|e| SerializedEvent::try_from(e).unwrap())
            .collect();
        let event_repo = MockRepo::with_events(Ok(ser_events));
        let (query, event_list) = MockQuery::new();
        let event_store = PersistedEventStore::new_event_store(event_repo);
        let query_replay = QueryReplay::new(event_store, query);
        query_replay.replay(AGGREGATE_ID).await.unwrap();

        let events = event_list.lock().unwrap().to_owned();
        assert_events_eq(events, expected_events);
    }

    fn assert_events_eq(
        expected: Vec<EventEnvelope<MyAggregate>>,
        found: Vec<EventEnvelope<MyAggregate>>,
    ) {
        assert_eq!(expected.len(), found.len());
        for i in 0..expected.len() {
            let ex = expected.get(i).unwrap();
            let f = found.get(i).unwrap();
            assert_eq!(ex.aggregate_id, f.aggregate_id);
            assert_eq!(ex.sequence, f.sequence);
            assert_eq!(ex.payload, f.payload);
            assert_eq!(ex.metadata, f.metadata);
        }
    }
}
