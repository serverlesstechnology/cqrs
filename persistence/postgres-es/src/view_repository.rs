use std::marker::PhantomData;

use cqrs_es::persist::{PersistenceError, ViewContext, ViewRepository};
use cqrs_es::{Aggregate, View};
use sqlx::postgres::PgRow;
use sqlx::{AssertSqlSafe, Pool, Postgres, Row, SqlSafeStr, SqlStr};

use crate::error::PostgresAggregateError;

/// A postgres backed query repository for use in backing a `GenericQuery`.
pub struct PostgresViewRepository<V, A> {
    insert_sql: SqlStr,
    update_sql: SqlStr,
    select_sql: SqlStr,
    pool: Pool<Postgres>,
    _phantom: PhantomData<(V, A)>,
}

impl<V, A> PostgresViewRepository<V, A>
where
    V: View<A>,
    A: Aggregate,
{
    /// Creates a new `PostgresViewRepository` that will store serialized views in a Postgres table named
    /// identically to the `view_name` value provided. This table should be created by the user
    /// before using this query repository (see `/db/init.sql` sql initialization file).
    ///
    /// ```
    /// # use cqrs_es::doc::MyAggregate;
    /// # use cqrs_es::persist::doc::MyView;
    /// use sqlx::{Pool, Postgres};
    /// use postgres_es::PostgresViewRepository;
    ///
    /// fn configure_view_repo(pool: Pool<Postgres>) -> PostgresViewRepository<MyView,MyAggregate> {
    ///     PostgresViewRepository::new("my_view_table", pool)
    /// }
    /// ```
    pub fn new(view_name: impl SqlSafeStr, pool: Pool<Postgres>) -> Self {
        let view_sql_str = view_name.into_sql_str();
        let insert_sql = AssertSqlSafe(format!(
            "INSERT INTO {} (payload, version, view_id) VALUES ( $1, $2, $3 )",
            view_sql_str.as_str()
        ))
        .into_sql_str();
        let update_sql = AssertSqlSafe(format!(
            "UPDATE {} SET payload= $1 , version= $2 WHERE view_id= $3 AND version= $4",
            view_sql_str.as_str()
        ))
        .into_sql_str();
        let select_sql = AssertSqlSafe(format!(
            "SELECT version,payload FROM {} WHERE view_id= $1",
            view_sql_str.as_str()
        ))
        .into_sql_str();
        Self {
            insert_sql,
            update_sql,
            select_sql,
            pool,
            _phantom: PhantomData,
        }
    }
}

impl<V, A> ViewRepository<V, A> for PostgresViewRepository<V, A>
where
    V: View<A>,
    A: Aggregate,
{
    async fn load(&self, view_id: &str) -> Result<Option<V>, PersistenceError> {
        let row: Option<PgRow> = sqlx::query(self.select_sql.clone())
            .bind(view_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(PostgresAggregateError::from)?;
        match row {
            None => Ok(None),
            Some(row) => {
                let view = serde_json::from_value(row.get("payload"))?;
                Ok(Some(view))
            }
        }
    }

    async fn load_with_context(
        &self,
        view_id: &str,
    ) -> Result<Option<(V, ViewContext)>, PersistenceError> {
        let row: Option<PgRow> = sqlx::query(self.select_sql.clone())
            .bind(view_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(PostgresAggregateError::from)?;
        match row {
            None => Ok(None),
            Some(row) => {
                let version = row.get("version");
                let view = serde_json::from_value(row.get("payload"))?;
                let view_context = ViewContext::new(view_id.to_string(), version);
                Ok(Some((view, view_context)))
            }
        }
    }

    async fn update_view(&self, view: V, context: ViewContext) -> Result<(), PersistenceError> {
        let sql = match context.version {
            0 => self.insert_sql.clone(),
            _ => self.update_sql.clone(),
        };
        let version = context.version + 1;
        let payload = serde_json::to_value(&view).map_err(PostgresAggregateError::from)?;
        let rows_affected = sqlx::query(sql)
            .bind(payload)
            .bind(version)
            .bind(context.view_instance_id)
            .bind(context.version)
            .execute(&self.pool)
            .await
            .map_err(PostgresAggregateError::from)?
            .rows_affected();
        if rows_affected < 1 {
            return Err(PersistenceError::OptimisticLockError);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::testing::tests::{
        Created, TestAggregate, TestEvent, TestView, TEST_CONNECTION_STRING,
    };
    use crate::{default_postgress_pool, PostgresViewRepository};
    use cqrs_es::persist::{ViewContext, ViewRepository};

    #[tokio::test]
    async fn test_valid_view_repository() {
        let pool = default_postgress_pool(TEST_CONNECTION_STRING).await;
        let repo =
            PostgresViewRepository::<TestView, TestAggregate>::new("test_view", pool.clone());
        let test_view_id = uuid::Uuid::new_v4().to_string();

        let view = TestView {
            events: vec![TestEvent::Created(Created {
                id: "just a test event for this view".to_string(),
            })],
        };
        repo.update_view(view.clone(), ViewContext::new(test_view_id.to_string(), 0))
            .await
            .unwrap();
        let (found, context) = repo
            .load_with_context(&test_view_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found, view);
        let found = repo.load(&test_view_id).await.unwrap().unwrap();
        assert_eq!(found, view);

        let updated_view = TestView {
            events: vec![TestEvent::Created(Created {
                id: "a totally different view".to_string(),
            })],
        };
        repo.update_view(updated_view.clone(), context)
            .await
            .unwrap();
        let found_option = repo.load(&test_view_id).await.unwrap();
        let found = found_option.unwrap();

        assert_eq!(found, updated_view);
    }
}
