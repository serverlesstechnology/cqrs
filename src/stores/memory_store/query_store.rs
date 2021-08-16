use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{
        Arc,
        RwLock,
    },
};

use crate::{
    aggregates::IAggregate,
    commands::ICommand,
    errors::AggregateError,
    events::IEvent,
    queries::{
        IQuery,
        QueryContext,
    },
    stores::IQueryStore,
};

type LockedQueryContextMap<C, E, Q> =
    RwLock<HashMap<String, QueryContext<C, E, Q>>>;

///  Simple memory store only useful for testing purposes
pub struct QueryStore<
    C: ICommand,
    E: IEvent,
    A: IAggregate<C, E>,
    Q: IQuery<C, E>,
> {
    events: Arc<LockedQueryContextMap<C, E, Q>>,
    _phantom: PhantomData<A>,
}

impl<
        C: ICommand,
        E: IEvent,
        A: IAggregate<C, E>,
        Q: IQuery<C, E>,
    > Default for QueryStore<C, E, A, Q>
{
    fn default() -> Self {
        Self {
            events: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<
        C: ICommand,
        E: IEvent,
        A: IAggregate<C, E>,
        Q: IQuery<C, E>,
    > QueryStore<C, E, A, Q>
{
    /// Get a shared copy of the events stored within the event store.
    pub fn get_events(&self) -> Arc<LockedQueryContextMap<C, E, Q>> {
        Arc::clone(&self.events)
    }

    fn load_query(
        &self,
        aggregate_id: &str,
    ) -> Option<QueryContext<C, E, Q>> {
        // uninteresting unwrap: this will not be used in production,
        // for tests only
        let event_map = self.events.read().unwrap();

        match event_map.get(aggregate_id) {
            None => None,
            Some(x) => Some(x.clone()),
        }
    }
}

impl<
        C: ICommand,
        E: IEvent,
        A: IAggregate<C, E>,
        Q: IQuery<C, E>,
    > IQueryStore<C, E, A, Q> for QueryStore<C, E, A, Q>
{
    /// loads the query
    fn load(
        &mut self,
        aggregate_id: &str,
    ) -> Result<QueryContext<C, E, Q>, AggregateError> {
        match self.load_query(aggregate_id) {
            None => {
                Err(AggregateError::new(
                    format!(
                        "Could not find aggregate '{}'",
                        aggregate_id
                    )
                    .as_str(),
                ))
            },
            Some(context) => Ok(context),
        }
    }

    /// commits the query
    fn commit(
        &mut self,
        context: QueryContext<C, E, Q>,
    ) -> Result<(), AggregateError> {
        let id = context.aggregate_id.clone();

        // uninteresting unwrap: this is not a struct for production
        // use
        let mut event_map = self.events.write().unwrap();
        event_map.insert(id, context);

        Ok(())
    }
}
