use std::collections::HashMap;

use async_trait::async_trait;

use crate::aggregate::Aggregate;
use crate::event::EventEnvelope;
use crate::AggregateError;

/// The abstract central source for loading past events and committing new
/// events.
#[async_trait]
pub trait EventStore<A>: Send + Sync
where
    A: Aggregate,
{
    /// Provides the current state of an aggregate along with surrounding
    /// context. This is used by the
    /// [CqrsFramework](struct.CqrsFramework.html) when loading an aggregate
    /// in order to handle incoming commands.
    type AC: AggregateContext<A>;

    /// Load all events for a particular `aggregate_id`
    async fn load_events(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError<A::Error>>;
    /// Load aggregate at current state
    async fn load_aggregate(
        &self,
        aggregate_id: &str,
    ) -> Result<Self::AC, AggregateError<A::Error>>;
    /// Commit new events
    async fn commit(
        &self,
        events: Vec<A::Event>,
        context: Self::AC,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<EventEnvelope<A>>, AggregateError<A::Error>>;
}

/// Returns the aggregate as well as the context around it.
/// This is used internally within an `EventStore` to persist an aggregate
/// instance and events with the correct context after it has been loaded and
/// modified.
pub trait AggregateContext<A>
where
    A: Aggregate,
{
    /// The aggregate instance with all state loaded.
    fn aggregate(&self) -> &A;
}
