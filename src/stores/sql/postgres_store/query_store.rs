use std::marker::PhantomData;

use postgres::Client;

use crate::{
    AggregateError,
    IAggregate,
    IQuery,
    IQueryStore,
    QueryContext,
};

use super::constants::{
    INSERT_QUERY,
    SELECT_QUERY,
    UPDATE_QUERY,
};

/// This provides a simple query repository that can be used both to
/// return deserialized views and to act as a query processor.
pub struct QueryStore<Q, A>
where
    Q: IQuery<A>,
    A: IAggregate, {
    conn: Client,
    _phantom: PhantomData<(Q, A)>,
}

impl<Q, A> QueryStore<Q, A>
where
    Q: IQuery<A>,
    A: IAggregate,
{
    /// Creates a new `QueryStore` that will store its'
    /// views in the table named identically to the `query_name`
    /// value provided. This table should be created by the user
    /// previously (see `/db/init.sql`).
    #[must_use]
    pub fn new(conn: Client) -> Self {
        QueryStore {
            conn,
            _phantom: PhantomData,
        }
    }
}

impl<Q, A> IQueryStore<Q, A> for QueryStore<Q, A>
where
    Q: IQuery<A>,
    A: IAggregate,
{
    fn load(
        &mut self,
        aggregate_id: &str,
    ) -> Result<QueryContext<Q, A>, AggregateError> {
        let agg_type = A::aggregate_type();
        let id = aggregate_id.to_string();
        let query_type = Q::query_type();

        let result = match self.conn.query(
            SELECT_QUERY,
            &[&agg_type, &id, &query_type],
        ) {
            Ok(x) => x,
            Err(e) => {
                return Err(AggregateError::new(
                    e.to_string().as_str(),
                ));
            },
        };

        match result.iter().next() {
            Some(row) => {
                let version = row.get(0);

                match serde_json::from_value(row.get(1)) {
                    Ok(payload) => {
                        Ok(QueryContext {
                            aggregate_id: id,
                            version,
                            payload,
                            _phantom: PhantomData,
                        })
                    },
                    Err(e) => {
                        Err(AggregateError::new(
                            e.to_string().as_str(),
                        ))
                    },
                }
            },
            None => {
                Ok(QueryContext {
                    aggregate_id: id,
                    version: 0,
                    payload: Default::default(),
                    _phantom: PhantomData,
                })
            },
        }
    }

    fn commit(
        &mut self,
        context: QueryContext<Q, A>,
    ) -> Result<(), AggregateError> {
        let agg_type = A::aggregate_type();
        let id = context.aggregate_id.as_str();
        let query_type = Q::query_type();

        let sql = match context.version {
            0 => INSERT_QUERY,
            _ => UPDATE_QUERY,
        };

        let version = context.version + 1;

        // let query_instance_id = &self.query_instance_id;
        let payload = match serde_json::to_value(&context.payload) {
            Ok(x) => x,
            Err(e) => {
                return Err(AggregateError::new(
                    format!(
                        "unable to serialize the payload of query \
                         '{}' with id: '{}', error: {}",
                        &query_type, &id, e,
                    )
                    .as_str(),
                ));
            },
        };

        match self.conn.execute(
            sql,
            &[
                &agg_type,
                &id,
                &query_type,
                &version,
                &payload,
            ],
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                return Err(AggregateError::new(
                    e.to_string().as_str(),
                ));
            },
        }
    }
}
