use crate::{
    aggregates::IAggregate,
    commands::ICommand,
    errors::AggregateError,
    events::{
        EventContext,
        IEvent,
        IEventDispatcher,
    },
    queries::{
        IQuery,
        QueryContext,
    },
};

/// The abstract central source for loading and committing
/// queries.
///
/// # Examples
/// ```rust
/// ```
pub trait IQueryStore<
    C: ICommand,
    E: IEvent,
    A: IAggregate<C, E>,
    Q: IQuery<C, E>,
> {
    /// loads the query
    fn load(
        &mut self,
        aggregate_id: &str,
    ) -> Result<QueryContext<C, E, Q>, AggregateError>;

    /// commits the query
    fn commit(
        &mut self,
        context: QueryContext<C, E, Q>,
    ) -> Result<(), AggregateError>;

    /// Used to apply committed events to a query.
    fn apply_events(
        &mut self,
        aggregate_id: &str,
        events: &[EventContext<C, E>],
    ) -> Result<(), AggregateError> {
        match self.load(aggregate_id) {
            Ok(mut context) => {
                for event in events {
                    context.payload.update(event);
                }
                self.commit(context)
            },
            Err(e) => Err(e),
        }
    }
}

impl<
        C: ICommand,
        E: IEvent,
        A: IAggregate<C, E>,
        Q: IQuery<C, E>,
    > IEventDispatcher<C, E> for dyn IQueryStore<C, E, A, Q>
{
    fn dispatch(
        &mut self,
        aggregate_id: &str,
        events: &[EventContext<C, E>],
    ) -> Result<(), AggregateError> {
        self.apply_events(aggregate_id, &events)
    }
}
