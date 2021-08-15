use crate::{
    aggregates::IAggregate,
    errors::AggregateError,
    events::EventContext,
};

/// Each CQRS platform should have one or more `QueryProcessor`s where
/// it will distribute committed events, it is the responsibility of
/// the `QueryProcessor` to update any interested queries.
pub trait IQueryProcessor<A: IAggregate> {
    /// Events will be dispatched here immediately after being
    /// committed for the downstream queries to be updated.
    fn dispatch(
        &mut self,
        aggregate_id: &str,
        events: &[EventContext<A>],
    ) -> Result<(), AggregateError>;
}
