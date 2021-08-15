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
pub trait IQueryStore<Q, A>: IQueryProcessor<A>
where
    Q: IQuery<A>,
    A: IAggregate, {
    /// loads the query
    fn load(
        &mut self,
        query_instance_id: String,
    ) -> Result<QueryContext<Q, A>, AggregateError>;

    /// commits the query
    fn commit(
        &mut self,
        query_context: QueryContext<Q, A>,
    ) -> Result<(), AggregateError>;

    /// Used to apply committed events to a query.
    fn apply_events(
        &mut self,
        query_instance_id: &str,
        events: &[EventContext<A>],
    ) -> Result<(), AggregateError> {
        match self.load(query_instance_id.to_string()) {
            Ok(mut view_context) => {
                for event in events {
                    view_context.query.update(event);
                }
                self.commit(view_context)
            },
            Err(e) => Err(e),
        }
    }
}
