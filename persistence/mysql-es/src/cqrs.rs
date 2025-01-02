use cqrs_es::persist::PersistedEventStore;
use cqrs_es::{Aggregate, CqrsFramework, Query};

use crate::{MysqlCqrs, MysqlEventRepository};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};

/// A convenience building a simple connection pool for MySql database.
pub async fn default_mysql_pool(connection_string: &str) -> Pool<MySql> {
    MySqlPoolOptions::new()
        .max_connections(10)
        .connect(connection_string)
        .await
        .expect("unable to connect to database")
}

/// A convenience method for building a simple connection pool for MySql.
/// A connection pool is needed for both the event and view repositories.
///
/// ```
/// use sqlx::{MySql, Pool};
/// use mysql_es::default_mysql_pool;
///
/// # async fn configure_pool() {
/// let connection_string = "mysql://test_user:test_pass@localhost:3306/test";
/// let pool: Pool<MySql> = default_mysql_pool(connection_string).await;
/// # }
/// ```
pub fn mysql_cqrs<A>(
    pool: Pool<MySql>,
    query_processor: Vec<Box<dyn Query<A>>>,
    services: A::Services,
) -> MysqlCqrs<A>
where
    A: Aggregate,
{
    let repo = MysqlEventRepository::new(pool);
    let store = PersistedEventStore::new_event_store(repo);
    CqrsFramework::new(store, query_processor, services)
}

/// A convenience function for creating a CqrsFramework using a snapshot store.
pub fn mysql_snapshot_cqrs<A>(
    pool: Pool<MySql>,
    query_processor: Vec<Box<dyn Query<A>>>,
    snapshot_size: usize,
    services: A::Services,
) -> MysqlCqrs<A>
where
    A: Aggregate,
{
    let repo = MysqlEventRepository::new(pool);
    let store = PersistedEventStore::new_snapshot_store(repo, snapshot_size);
    CqrsFramework::new(store, query_processor, services)
}

/// A convenience function for creating a CqrsFramework using an aggregate store.
pub fn mysql_aggregate_cqrs<A>(
    pool: Pool<MySql>,
    query_processor: Vec<Box<dyn Query<A>>>,
    services: A::Services,
) -> MysqlCqrs<A>
where
    A: Aggregate,
{
    let repo = MysqlEventRepository::new(pool);
    let store = PersistedEventStore::new_aggregate_store(repo);
    CqrsFramework::new(store, query_processor, services)
}

#[cfg(test)]
mod test {
    use crate::testing::tests::{
        TestAggregate, TestQueryRepository, TestServices, TestView, TEST_CONNECTION_STRING,
    };
    use crate::{default_mysql_pool, mysql_cqrs, MysqlViewRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_valid_cqrs_framework() {
        let pool = default_mysql_pool(TEST_CONNECTION_STRING).await;
        let repo = MysqlViewRepository::<TestView, TestAggregate>::new("test_view", pool.clone());
        let query = TestQueryRepository::new(Arc::new(repo));
        let _ps = mysql_cqrs(pool, vec![Box::new(query)], TestServices);
    }
}
