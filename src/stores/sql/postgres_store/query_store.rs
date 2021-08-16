use std::marker::PhantomData;

use postgres::Client;

use crate::{
    aggregates::IAggregate,
    commands::ICommand,
    errors::AggregateError,
    events::IEvent,
    queries::{
        IQuery,
        QueryContext,
    },
    stores::IQueryStore,
};

use super::constants::{
    INSERT_QUERY,
    SELECT_QUERY,
    UPDATE_QUERY,
};

/// This provides a simple query repository that can be used both to
/// return deserialized views and to act as a query processor.
pub struct QueryStore<
    C: ICommand,
    E: IEvent,
    A: IAggregate<C, E>,
    Q: IQuery<C, E>,
> {
    conn: Client,
    _phantom: PhantomData<(C, E, A, Q)>,
}

impl<
        C: ICommand,
        E: IEvent,
        A: IAggregate<C, E>,
        Q: IQuery<C, E>,
    > QueryStore<C, E, A, Q>
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

impl<
        C: ICommand,
        E: IEvent,
        A: IAggregate<C, E>,
        Q: IQuery<C, E>,
    > IQueryStore<C, E, A, Q> for QueryStore<C, E, A, Q>
{
    fn load(
        &mut self,
        aggregate_id: &str,
    ) -> Result<QueryContext<C, E, Q>, AggregateError> {
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
                        Ok(QueryContext::new(id, version, payload))
                    },
                    Err(e) => {
                        Err(AggregateError::new(
                            e.to_string().as_str(),
                        ))
                    },
                }
            },
            None => {
                Ok(QueryContext::new(
                    id,
                    0,
                    Default::default(),
                ))
            },
        }
    }

    fn commit(
        &mut self,
        context: QueryContext<C, E, Q>,
    ) -> Result<(), AggregateError> {
        let agg_type = A::aggregate_type();
        let id = context.aggregate_id.as_str();
        let query_type = Q::query_type();
        let version = context.version;

        let sql = match version {
            1 => INSERT_QUERY,
            _ => UPDATE_QUERY,
        };

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
