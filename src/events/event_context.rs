use log::trace;
use std::{
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
};

use crate::commands::ICommand;

use super::i_event::IEvent;

/// `EventContext` is a data structure that encapsulates an event
/// with along with it's pertinent information. All of the associated
/// data will be transported and persisted together.
///
/// Within any system an event must be unique based on its'
/// `aggregate_type`, `aggregate_id` and `sequence`.
#[derive(Debug, PartialEq, Clone)]
pub struct EventContext<C: ICommand, E: IEvent> {
    /// The id of the aggregate instance.
    pub aggregate_id: String,

    /// The sequence number for an aggregate instance.
    pub sequence: usize,

    /// The event payload with all business information.
    pub payload: E,

    /// Additional metadata for use in auditing, logging or debugging
    /// purposes.
    pub metadata: HashMap<String, String>,

    /// phantom data
    _phantom: PhantomData<C>,
}

impl<C: ICommand, E: IEvent> EventContext<C, E> {
    /// Constructor
    pub fn new(
        aggregate_id: String,
        sequence: usize,
        payload: E,
        metadata: HashMap<String, String>,
    ) -> Self {
        let x = Self {
            aggregate_id,
            sequence,
            payload,
            metadata,
            _phantom: PhantomData,
        };

        trace!("Created new {:?}", x,);

        x
    }
}
