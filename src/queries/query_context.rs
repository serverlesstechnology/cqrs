use log::trace;
use std::{
    fmt::Debug,
    marker::PhantomData,
};

use crate::{
    commands::ICommand,
    events::IEvent,
};

use super::i_query::IQuery;

/// Returns the query and context around it that is needed when
/// committing in a query store implementation.
#[derive(Debug, PartialEq, Clone)]
pub struct QueryContext<C: ICommand, E: IEvent, Q: IQuery<C, E>> {
    /// The id of the aggregate instance.
    pub aggregate_id: String,

    /// The current version number for this query instance.
    pub version: i64,

    /// The current state of the query instance.
    pub payload: Q,

    _phantom: PhantomData<(C, E)>,
}

impl<C: ICommand, E: IEvent, Q: IQuery<C, E>> QueryContext<C, E, Q> {
    /// Constructor
    pub fn new(
        aggregate_id: String,
        version: i64,
        payload: Q,
    ) -> Self {
        let x = Self {
            aggregate_id,
            version,
            payload,
            _phantom: PhantomData,
        };

        trace!("Created new {:?}", x,);

        x
    }
}
