use std::marker::PhantomData;

use cqrs_es::persist::{PersistenceError, ViewContext, ViewRepository};
use cqrs_es::{Aggregate, View};
use sqlx::mysql::MySqlRow;
use sqlx::{AssertSqlSafe, MySql, Pool, Row, SqlSafeStr, SqlStr};

use crate::error::MysqlAggregateError;

/// A mysql backed query repository for use in backing a `GenericQuery`.
pub struct MysqlViewRepository<V, A> {
    insert_sql: SqlStr,
    update_sql: SqlStr,
    select_sql: SqlStr,
    pool: Pool<MySql>,
    _phantom: PhantomData<(V, A)>,
}

impl<V, A> MysqlViewRepository<V, A>
where
    V: View<A>,
    A: Aggregate,
{
    /// Creates a new `MysqlViewRepository` that will store serialized views in a MySql table named
    /// identically to the `view_name` value provided. This table should be created by the user
    /// before using this query repository (see `/db/init.sql` sql initialization file).
    ///
    /// ```
    /// # use cqrs_es::doc::MyAggregate;
    /// # use cqrs_es::persist::doc::MyView;
    /// use sqlx::{MySql, Pool};
    /// use mysql_es::MysqlViewRepository;
    ///
    /// fn configure_view_repo(pool: Pool<MySql>) -> MysqlViewRepository<MyView,MyAggregate> {
    ///     MysqlViewRepository::new("my_view_table", pool)
    /// }
    /// ```
    pub fn new(view_name: impl SqlSafeStr, pool: Pool<MySql>) -> Self {
        let view_sql_str = view_name.into_sql_str();
        let insert_sql = AssertSqlSafe(format!(
            "INSERT INTO {} (payload, version, view_id) VALUES ( ?, ?, ? )",
            view_sql_str.as_str()
        ))
        .into_sql_str();
        let update_sql = AssertSqlSafe(format!(
            "UPDATE {} SET payload= ? , version= ? WHERE view_id= ?",
            view_sql_str.as_str()
        ))
        .into_sql_str();
        let select_sql = AssertSqlSafe(format!(
            "SELECT version,payload FROM {} WHERE view_id= ?",
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

impl<V, A> ViewRepository<V, A> for MysqlViewRepository<V, A>
where
    V: View<A>,
    A: Aggregate,
{
    async fn load(&self, view_id: &str) -> Result<Option<V>, PersistenceError> {
        let row: Option<MySqlRow> = sqlx::query(self.select_sql.clone())
            .bind(view_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(MysqlAggregateError::from)?;
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
        let row: Option<MySqlRow> = sqlx::query(self.select_sql.clone())
            .bind(view_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(MysqlAggregateError::from)?;
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
        let payload = serde_json::to_value(&view).map_err(MysqlAggregateError::from)?;
        sqlx::query(sql)
            .bind(payload)
            .bind(version)
            .bind(context.view_instance_id)
            .execute(&self.pool)
            .await
            .map_err(MysqlAggregateError::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::testing::tests::{
        Created, TestAggregate, TestEvent, TestView, TEST_CONNECTION_STRING,
    };
    use crate::{default_mysql_pool, MysqlViewRepository};
    use cqrs_es::persist::{ViewContext, ViewRepository};

    #[tokio::test]
    async fn test_valid_view_repository() {
        let pool = default_mysql_pool(TEST_CONNECTION_STRING).await;
        let repo = MysqlViewRepository::<TestView, TestAggregate>::new("test_view", pool.clone());
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
