use cqrs_es::persist::PersistedEventStore;
use cqrs_es::{Aggregate, CqrsFramework, Query};

use crate::{PostgresCqrs, PostgresEventRepository};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

/// A convenience method for building a simple connection pool for PostgresDb.
/// A connection pool is needed for both the event and view repositories.
///
/// ```
/// use sqlx::{Pool, Postgres};
/// use postgres_es::default_postgress_pool;
///
/// # async fn configure_pool() {
/// let connection_string = "postgresql://test_user:test_pass@localhost:5432/test";
/// let pool: Pool<Postgres> = default_postgress_pool(connection_string).await;
/// # }
/// ```
pub async fn default_postgress_pool(connection_string: &str) -> Pool<Postgres> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(connection_string)
        .await
        .expect("unable to connect to database")
}

/// A convenience function for creating a CqrsFramework from a database connection pool
/// and queries.
pub fn postgres_cqrs<A>(
    pool: Pool<Postgres>,
    query_processor: Vec<Box<dyn Query<A>>>,
    services: A::Services,
) -> PostgresCqrs<A>
where
    A: Aggregate,
{
    let repo = PostgresEventRepository::new(pool);
    let store = PersistedEventStore::new_event_store(repo);
    CqrsFramework::new(store, query_processor, services)
}

/// A convenience function for creating a CqrsFramework using a snapshot store.
pub fn postgres_snapshot_cqrs<A>(
    pool: Pool<Postgres>,
    query_processor: Vec<Box<dyn Query<A>>>,
    snapshot_size: usize,
    services: A::Services,
) -> PostgresCqrs<A>
where
    A: Aggregate,
{
    let repo = PostgresEventRepository::new(pool);
    let store = PersistedEventStore::new_snapshot_store(repo, snapshot_size);
    CqrsFramework::new(store, query_processor, services)
}

/// A convenience function for creating a CqrsFramework using an aggregate store.
pub fn postgres_aggregate_cqrs<A>(
    pool: Pool<Postgres>,
    query_processor: Vec<Box<dyn Query<A>>>,
    services: A::Services,
) -> PostgresCqrs<A>
where
    A: Aggregate,
{
    let repo = PostgresEventRepository::new(pool);
    let store = PersistedEventStore::new_aggregate_store(repo);
    CqrsFramework::new(store, query_processor, services)
}

#[cfg(test)]
mod test {
    use crate::testing::tests::{
        TestAggregate, TestQueryRepository, TestServices, TestView, TEST_CONNECTION_STRING,
    };
    use crate::{default_postgress_pool, postgres_cqrs, PostgresViewRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_valid_cqrs_framework() {
        let pool = default_postgress_pool(TEST_CONNECTION_STRING).await;
        let repo =
            PostgresViewRepository::<TestView, TestAggregate>::new("test_view", pool.clone());
        let query = TestQueryRepository::new(Arc::new(repo));
        let _ps = postgres_cqrs(pool, vec![Box::new(query)], TestServices);
    }
}
