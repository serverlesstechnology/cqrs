use std::marker::PhantomData;

use async_trait::async_trait;

use crate::persist::{PersistenceError, QueryContext, ViewRepository};
use crate::{Aggregate, EventEnvelope, Query, View};

/// A simple query and view repository. This is used both to act as a `Query` for processing events
/// and to return materialized views.
pub struct GenericQuery<R, V, A>
where
    R: ViewRepository<V, A>,
    V: View<A>,
    A: Aggregate,
{
    repo: R,
    error_handler: Option<Box<ErrorHandler>>,
    phantom: PhantomData<(V, A)>,
}

type ErrorHandler = dyn Fn(PersistenceError) + Send + Sync + 'static;

impl<R, V, A> GenericQuery<R, V, A>
where
    R: ViewRepository<V, A>,
    V: View<A>,
    A: Aggregate,
{
    /// Creates a new `GenericQuery` that will store serialized views in a Postgres table named
    /// identically to the `query_name` value provided. This table should be created by the user
    /// before using this query repository (see `/db/init.sql` sql initialization file).
    ///
    /// ```ignore
    /// let query = GenericQuery::<MyView, MyAggregate>::new("my_query", pool.clone());
    /// let store = ...
    /// let cqrs = CqrsFramework::new(store, vec![Box::new(query)]);
    /// ```
    #[must_use]
    pub fn new(repo: R) -> Self {
        GenericQuery {
            repo,
            error_handler: None,
            phantom: Default::default(),
        }
    }
    /// Allows the user to apply a custom error handler to the query.
    /// Queries are infallible and _should_ never cause errors,
    /// but programming errors or other technical problems
    /// might. Adding an error handler allows the user to choose whether to
    /// panic the application, log the error or otherwise register the issue.
    ///
    /// This is not required for usage but without an error handler any error encountered
    /// by the query repository will simply be ignored,
    /// so it is *strongly* recommended.
    ///
    /// _An error handler that panics on any error._
    /// ```ignore
    /// query.use_error_handler(Box::new(|e|panic!("{}",e)));
    /// ```
    pub fn use_error_handler(&mut self, error_handler: Box<ErrorHandler>) {
        self.error_handler = Some(error_handler);
    }

    /// Loads and deserializes a view based on the provided view id.
    /// Use this method to load a materialized view when requested by a user.
    ///
    /// This is an asynchronous method so don't forget to `await`.
    ///
    /// ```ignore
    /// let view = query.load("customer-B24DA0".to_string()).await;
    /// ```
    pub async fn load(&self, query_instance_id: String) -> Option<V> {
        match self.repo.load(&query_instance_id).await {
            Ok(option) => match option {
                None => None,
                Some((view, _)) => Some(view),
            },
            Err(e) => {
                self.handle_error(e.into());
                None
            }
        }
    }

    async fn load_mut(
        &self,
        query_instance_id: String,
    ) -> Result<(V, QueryContext), PersistenceError> {
        match self.repo.load(&query_instance_id).await? {
            None => {
                let view_context = QueryContext::new(query_instance_id, 0);
                Ok((Default::default(), view_context))
            }
            Some((view, context)) => Ok((view, context)),
        }
    }

    pub(crate) async fn apply_events(
        &self,
        query_instance_id: &str,
        events: &[EventEnvelope<A>],
    ) -> Result<(), PersistenceError> {
        let (mut view, view_context) = self.load_mut(query_instance_id.to_string()).await?;
        for event in events {
            view.update(event);
        }
        self.repo.update_view(view, view_context).await?;
        Ok(())
    }

    fn handle_error(&self, error: PersistenceError) {
        match &self.error_handler {
            None => {}
            Some(handler) => {
                (handler)(error);
            }
        }
    }
}

#[async_trait]
impl<R, V, A> Query<A> for GenericQuery<R, V, A>
where
    R: ViewRepository<V, A>,
    V: View<A>,
    A: Aggregate,
{
    async fn dispatch(&self, query_instance_id: &str, events: &[EventEnvelope<A>]) {
        match self
            .apply_events(&query_instance_id.to_string(), events)
            .await
        {
            Ok(_) => {}
            Err(err) => self.handle_error(err),
        };
    }
}
