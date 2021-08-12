use crate::{
    aggregates::Aggregate,
    events::EventEnvelope,
};

/// Each CQRS platform should have one or more `QueryProcessor`s where
/// it will distribute committed events, it is the responsibility of
/// the `QueryProcessor` to update any interested queries.
pub trait QueryProcessor<A: Aggregate> {
    /// Events will be dispatched here immediately after being
    /// committed for the downstream queries to be updated.
    fn dispatch(
        &self,
        aggregate_id: &str,
        events: &[EventEnvelope<A>],
    );
}
