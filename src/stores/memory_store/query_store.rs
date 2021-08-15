use std::{
    collections::HashMap,
    sync::{
        Arc,
        RwLock,
    },
};

use crate::{
    aggregates::IAggregate,
    errors::AggregateError,
    queries::{
        IQuery,
        QueryContext,
    },
};

use super::super::IQueryStore;

type LockedQueryContextMap<Q, A> =
    RwLock<HashMap<String, QueryContext<Q, A>>>;

///  Simple memory store only useful for testing purposes
pub struct QueryStore<Q, A>
where
    Q: IQuery<A>,
    A: IAggregate, {
    events: Arc<LockedQueryContextMap<Q, A>>,
}

impl<Q, A> Default for QueryStore<Q, A>
where
    Q: IQuery<A>,
    A: IAggregate,
{
    fn default() -> Self {
        let events = Default::default();
        QueryStore { events }
    }
}

impl<Q, A> QueryStore<Q, A>
where
    Q: IQuery<A>,
    A: IAggregate,
{
    /// Get a shared copy of the events stored within the event store.
    pub fn get_events(&self) -> Arc<LockedQueryContextMap<Q, A>> {
        Arc::clone(&self.events)
    }

    fn load_query(
        &self,
        aggregate_id: &str,
    ) -> Option<QueryContext<Q, A>> {
        // uninteresting unwrap: this will not be used in production,
        // for tests only
        let event_map = self.events.read().unwrap();

        match event_map.get(aggregate_id) {
            None => None,
            Some(x) => Some(x.clone()),
        }
    }
}

impl<Q, A> IQueryStore<Q, A> for QueryStore<Q, A>
where
    Q: IQuery<A>,
    A: IAggregate,
{
    /// loads the query
    fn load(
        &mut self,
        aggregate_id: &str,
    ) -> Result<QueryContext<Q, A>, AggregateError> {
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
        context: QueryContext<Q, A>,
    ) -> Result<(), AggregateError> {
        let id = context.aggregate_id.clone();

        // uninteresting unwrap: this is not a struct for production
        // use
        let mut event_map = self.events.write().unwrap();
        event_map.insert(id, context);

        Ok(())
    }
}
