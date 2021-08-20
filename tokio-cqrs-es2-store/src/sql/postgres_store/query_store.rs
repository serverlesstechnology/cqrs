use async_trait::async_trait;
use std::marker::PhantomData;

use sqlx::postgres::PgPool;

use cqrs_es2_core::{
    Error,
    EventContext,
    IAggregate,
    ICommand,
    IEvent,
    IQuery,
    QueryContext,
};

use crate::repository::{
    IEventDispatcher,
    IQueryStore,
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
    pool: PgPool,
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
    pub fn new(pool: PgPool) -> Self {
        QueryStore {
            pool,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<
        C: ICommand,
        E: IEvent,
        A: IAggregate<C, E>,
        Q: IQuery<C, E>,
    > IQueryStore<C, E, A, Q> for QueryStore<C, E, A, Q>
{
    async fn load(
        &mut self,
        aggregate_id: &str,
    ) -> Result<QueryContext<C, E, Q>, Error> {
        let agg_type = A::aggregate_type();
        let query_type = Q::query_type();

        let rows: Vec<(i64, serde_json::Value)> =
            match sqlx::query_as(SELECT_QUERY)
                .bind(&agg_type)
                .bind(&aggregate_id)
                .bind(&query_type)
                .fetch_all(&self.pool)
                .await
            {
                Ok(x) => x,
                Err(e) => {
                    return Err(Error::new(e.to_string().as_str()));
                },
            };

        if rows.len() == 0 {
            return Ok(QueryContext::new(
                aggregate_id.to_string(),
                0,
                Default::default(),
            ));
        }

        let row = rows[0].clone();

        let payload = match serde_json::from_value(row.1) {
            Ok(x) => x,
            Err(e) => {
                return Err(Error::new(e.to_string().as_str()));
            },
        };

        Ok(QueryContext::new(
            aggregate_id.to_string(),
            row.0,
            payload,
        ))
    }

    async fn commit(
        &mut self,
        context: QueryContext<C, E, Q>,
    ) -> Result<(), Error> {
        let agg_type = A::aggregate_type();
        let aggregate_id = context.aggregate_id.as_str();
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
                return Err(Error::new(
                    format!(
                        "unable to serialize the payload of query \
                         '{}' with id: '{}', error: {}",
                        &query_type, &aggregate_id, e,
                    )
                    .as_str(),
                ));
            },
        };

        match sqlx::query(sql)
            .bind(&agg_type)
            .bind(&aggregate_id)
            .bind(&query_type)
            .bind(version)
            .bind(&payload)
            .execute(&self.pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                return Err(Error::new(e.to_string().as_str()));
            },
        }
    }
}

#[async_trait]
impl<
        C: ICommand,
        E: IEvent,
        A: IAggregate<C, E>,
        Q: IQuery<C, E>,
    > IEventDispatcher<C, E> for QueryStore<C, E, A, Q>
{
    async fn dispatch(
        &mut self,
        aggregate_id: &str,
        events: &[EventContext<C, E>],
    ) -> Result<(), Error> {
        self.dispatch_events(aggregate_id, events)
            .await
    }
}
