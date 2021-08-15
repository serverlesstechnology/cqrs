use std::marker::PhantomData;

use super::super::aggregates::IAggregate;

use super::i_query::IQuery;

/// Returns the query and context around it that is needed when
/// committing in a query store implementation.
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

impl<Q, A> Clone for QueryContext<Q, A>
where
    Q: IQuery<A>,
    A: IAggregate,
{
    fn clone(&self) -> Self {
        QueryContext {
            query_instance_id: self.query_instance_id.clone(),
            query: self.query.clone(),
            version: self.version,
            _phantom: PhantomData::default(),
        }
    }
}
