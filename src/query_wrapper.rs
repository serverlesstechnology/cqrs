use crate::{Aggregate, EventEnvelope, Query};
use std::{future::Future, pin::Pin, sync::Arc};

type BoxFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

/// `QueryWrapper` hides the `Query<Foo>` data type.
pub struct QueryWrapper<A: Aggregate> {
    inner: Arc<dyn for<'a> Fn(&'a str, &'a [EventEnvelope<A>]) -> BoxFuture<'a> + Send + Sync>,
}

impl<A: Aggregate> QueryWrapper<A> {
    /// Create a new wrapper around the query.
    pub fn new<Q>(query: Q) -> Self
    where
        Q: Query<A> + Send + Sync + 'static,
    {
        let query = Arc::new(query);

        Self {
            inner: Arc::new(move |aggregate_id, events| {
                let q = Arc::clone(&query);
                Box::pin(async move {
                    q.dispatch(aggregate_id, events).await;
                })
            }),
        }
    }

    /// Dispatch an event for the typed `Query<Foo>`.
    pub async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<A>]) {
        (self.inner)(aggregate_id, events).await
    }
}
