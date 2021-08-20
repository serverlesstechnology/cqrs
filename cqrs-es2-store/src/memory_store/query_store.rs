use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{
        Arc,
        RwLock,
    },
};

use cqrs_es2_core::{
    Error,
    EventContext,
    IAggregate,
    ICommand,
    IEvent,
    IQuery,
    QueryContext,
};

use crate::repository::{
    IEventDispatcher,
    IQueryStore,
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
    ) -> Result<QueryContext<C, E, Q>, Error> {
        // uninteresting unwrap: this will not be used in production,
        // for tests only
        let event_map = self.events.read().unwrap();

        match event_map.get(aggregate_id) {
            None => {
                Ok(QueryContext::new(
                    aggregate_id.to_string(),
                    0,
                    Default::default(),
                ))
            },
            Some(x) => Ok(x.clone()),
        }
    }

    /// commits the query
    fn commit(
        &mut self,
        context: QueryContext<C, E, Q>,
    ) -> Result<(), Error> {
        let id = context.aggregate_id.clone();

        // uninteresting unwrap: this is not a struct for production
        // use
        let mut event_map = self.events.write().unwrap();
        event_map.insert(id, context);

        Ok(())
    }
}

impl<
        C: ICommand,
        E: IEvent,
        A: IAggregate<C, E>,
        Q: IQuery<C, E>,
    > IEventDispatcher<C, E> for QueryStore<C, E, A, Q>
{
    fn dispatch(
        &mut self,
        aggregate_id: &str,
        events: &[EventContext<C, E>],
    ) -> Result<(), Error> {
        self.dispatch_events(aggregate_id, events)
    }
}
