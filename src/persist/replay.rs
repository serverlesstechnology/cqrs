use std::marker::PhantomData;

use crate::persist::{PersistedEventRepository, PersistenceError, QueryErrorHandler};
use crate::{Aggregate, AggregateError, EventEnvelope, Query};

/// A utility for replaying committed events to a `Query`.
///
/// ```rust
/// use cqrs_es::doc::{MyAggregate, MyQuery, MyRepository};
/// use cqrs_es::persist::QueryReplay;
///
/// fn update(repo: MyRepository, query: MyQuery) {
///     let replay = QueryReplay::new(repo, query);
///     replay.replay_all();
/// }
/// ```
pub struct QueryReplay<R, Q, A>
where
    R: PersistedEventRepository,
    Q: Query<A>,
    A: Aggregate,
{
    repository: R,
    query: Q,
    error_handler: Option<Box<QueryErrorHandler>>,
    phantom_data: PhantomData<A>,
}

impl<R, Q, A> QueryReplay<R, Q, A>
where
    R: PersistedEventRepository,
    Q: Query<A>,
    A: Aggregate,
{
    /// Create a new replay utility using the provided event repository as the source and the
    /// query as the target.
    pub fn new(repository: R, query: Q) -> Self {
        Self {
            repository,
            query,
            error_handler: None,
            phantom_data: PhantomData::default(),
        }
    }

    /// Allows the user to apply a custom error handler to the query replay.
    ///
    /// _Example: An error handler that panics on any error._
    /// ```
    /// # use cqrs_es::doc::{MyAggregate, MyQuery, MyRepository};
    /// # use cqrs_es::persist::{GenericQuery, QueryReplay, ReplayStream};
    /// # fn config(mut replay: QueryReplay<MyRepository,MyQuery,MyAggregate>) {
    /// replay.use_error_handler(Box::new(|e|panic!("{}",e)));
    /// # }
    /// ```
    pub fn use_error_handler(&mut self, error_handler: Box<QueryErrorHandler>) {
        self.error_handler = Some(error_handler);
    }

    /// Replay the events of a single aggregate instance.
    pub async fn replay(&self, aggregate_id: &str) -> Result<(), AggregateError<A::Error>> {
        let mut stream = self.repository.stream_events::<A>(aggregate_id).await?;
        while let Some(event) = stream.next().await {
            self.apply(event).await;
        }
        Ok(())
    }

    /// Replay the events of all aggregate instances within the database.
    pub async fn replay_all(&self) -> Result<(), AggregateError<A::Error>> {
        let mut stream = self.repository.stream_all_events::<A>().await?;
        while let Some(event) = stream.next().await {
            self.apply(event).await;
        }
        Ok(())
    }

    async fn apply(&self, event: Result<EventEnvelope<A>, PersistenceError>) {
        match event {
            Ok(event) => {
                let aggregate_id = event.aggregate_id.clone();
                self.query.dispatch(&aggregate_id, &[event]).await;
            }
            Err(error) => {
                if let Some(handler) = &self.error_handler {
                    (handler)(error);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use crate::doc::{MyAggregate, MyEvents};
    use crate::persist::event_store::shared_test::MockRepo;
    use crate::persist::replay::QueryReplay;
    use crate::persist::SerializedEvent;
    use crate::{EventEnvelope, Query};

    #[derive(Debug)]
    struct MockQuery {
        events: Arc<Mutex<Vec<EventEnvelope<MyAggregate>>>>,
    }

    impl MockQuery {
        fn new() -> (Self, Arc<Mutex<Vec<EventEnvelope<MyAggregate>>>>) {
            let events: Arc<Mutex<Vec<EventEnvelope<MyAggregate>>>> = Arc::default();
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
            metadata: HashMap::default(),
        }];
        let ser_events: Vec<SerializedEvent> = expected_events
            .iter()
            .map(|e| SerializedEvent::try_from(e).unwrap())
            .collect();
        let event_repo = MockRepo::with_events(Ok(ser_events.clone()));
        let (query, event_list) = MockQuery::new();
        let query_replay = QueryReplay::new(event_repo, query);
        query_replay.replay(AGGREGATE_ID).await.unwrap();

        let events = event_list.lock().unwrap().to_owned();
        assert_events_eq(&events, &expected_events);

        // query all
        let event_repo = MockRepo::with_events(Ok(ser_events));
        let (query, event_list) = MockQuery::new();
        let query_replay = QueryReplay::new(event_repo, query);
        query_replay.replay_all().await.unwrap();

        let events = event_list.lock().unwrap().to_owned();
        assert_events_eq(&events, &expected_events);
    }

    fn assert_events_eq(
        expected: &[EventEnvelope<MyAggregate>],
        found: &[EventEnvelope<MyAggregate>],
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
