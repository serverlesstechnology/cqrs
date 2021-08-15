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
    /// The query ID of the query instance that has been loaded.
    pub query_instance_id: String,

    /// The current state of the query instance.
    pub query: Q,

    /// The current version number for this query instance.
    pub version: i64,

    _phantom: PhantomData<A>,
}
