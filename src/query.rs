use async_trait::async_trait;
use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::aggregate::Aggregate;
use crate::event::EventEnvelope;

/// Each CQRS platform should have one or more queries where it will distribute committed
/// events.
///
/// Some example of tasks that queries commonly provide:
/// - update materialized views
/// - publish events to messaging service
/// - trigger a command on another aggregate
#[async_trait]
pub trait Query<A: Aggregate>: Send + Sync {
    /// Events will be dispatched here immediately after being committed.
    async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<A>]);
}

/// A `View` represents a materialized view, generally serialized for persistence, that is updated by a query.
/// This a read element in a CQRS system.
///
pub trait View<A: Aggregate>: Debug + Default + Serialize + DeserializeOwned + Send + Sync {
    /// Each implemented view is responsible for updating its state based on events passed via
    /// this method.
    fn update(&mut self, event: &EventEnvelope<A>);
}
