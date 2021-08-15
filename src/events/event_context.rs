use std::{
    collections::HashMap,
    fmt::Debug,
};

use crate::aggregates::IAggregate;

/// `EventContext` is a data structure that encapsulates an event
/// with along with it's pertinent information. All of the associated
/// data will be transported and persisted together.
///
/// Within any system an event must be unique based on its'
/// `aggregate_type`, `aggregate_id` and `sequence`.
#[derive(Debug, Clone)]
pub struct EventContext<A: IAggregate> {
    /// The id of the aggregate instance.
    pub aggregate_id: String,

    /// The sequence number for an aggregate instance.
    pub sequence: usize,

    /// The event payload with all business information.
    pub payload: A::Event,

    /// Additional metadata for use in auditing, logging or debugging
    /// purposes.
    pub metadata: HashMap<String, String>,
}
