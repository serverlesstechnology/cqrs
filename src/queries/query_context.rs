use std::{
    fmt::Debug,
    marker::PhantomData,
};

use super::super::aggregates::IAggregate;

use super::i_query::IQuery;

/// Returns the query and context around it that is needed when
/// committing in a query store implementation.
#[derive(Debug, Clone)]
pub struct QueryContext<Q, A>
where
    Q: IQuery<A>,
    A: IAggregate, {
    /// The id of the aggregate instance.
    pub aggregate_id: String,

    /// The current version number for this query instance.
    pub version: i64,

    /// The current state of the query instance.
    pub payload: Q,

    /// phantom data for aggregate type
    pub _phantom: PhantomData<A>,
}
