use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::aggregate::Aggregate;
use crate::event::{DomainEvent, MessageEnvelope};

/// Each CQRS platform should have one or more `QueryProcessor`s where it will distribute committed
/// events, it is the responsibility of the `QueryProcessor` to update any interested
/// queries.
pub trait QueryProcessor<A, E>
    where E: DomainEvent<A>,
          A: Aggregate
{
    /// Events will be dispatched here immediately after being committed for the downstream queries
    /// to be updated.
    fn dispatch(&self, aggregate_id: &str, events: &[MessageEnvelope<A, E>]);
}

/// Downstream `Query`s are the read elements in a CQRS system. As events are emitted these queries
/// , or views, are updated to reflect the currente state of the system.
///
/// These are regularly a serialized view, usually stored in a standard database, but could
/// also include messaging platform or other asynchronous eventually consistent platforms.
pub trait Query<A, E>: Debug + Default + Serialize + DeserializeOwned + Default
    where E: DomainEvent<A>,
          A: Aggregate
{
    /// Each implemented query is responsible for updating its stated based on events passed via
    /// this method.
    fn update(&mut self, event: &MessageEnvelope<A, E>);
}




