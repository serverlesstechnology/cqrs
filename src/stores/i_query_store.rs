use crate::{
    aggregates::IAggregate,
    errors::AggregateError,
    events::EventContext,
    queries::{
        IQuery,
        IQueryProcessor,
        QueryContext,
    },
};

/// The abstract central source for loading and committing
/// queries.
///
/// # Examples
/// ```rust
/// ```
pub trait IQueryStore<Q, A>
where
    Q: IQuery<A>,
    A: IAggregate, {
    /// loads the query
    fn load(
        &mut self,
        aggregate_id: &str,
    ) -> Result<QueryContext<Q, A>, AggregateError>;

    /// commits the query
    fn commit(
        &mut self,
        context: QueryContext<Q, A>,
    ) -> Result<(), AggregateError>;

    /// Used to apply committed events to a query.
    fn apply_events(
        &mut self,
        aggregate_id: &str,
        events: &[EventContext<A>],
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

impl<Q, A> IQueryProcessor<A> for dyn IQueryStore<Q, A>
where
    Q: IQuery<A>,
    A: IAggregate,
{
    fn dispatch(
        &mut self,
        aggregate_id: &str,
        events: &[EventContext<A>],
    ) -> Result<(), AggregateError> {
        self.apply_events(aggregate_id, &events)
    }
}
