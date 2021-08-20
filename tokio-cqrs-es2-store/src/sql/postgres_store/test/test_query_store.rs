use sqlx::postgres::PgPoolOptions;

use cqrs_es2_core::{
    example_impl::*,
    Error,
    QueryContext,
};

use crate::{
    postgres_store::QueryStore,
    repository::IQueryStore,
};

use super::common::*;

type ThisQueryStore = QueryStore<
    CustomerCommand,
    CustomerEvent,
    Customer,
    CustomerContactQuery,
>;

type ThisQueryContext = QueryContext<
    CustomerCommand,
    CustomerEvent,
    CustomerContactQuery,
>;

async fn commit_and_load_queries() -> Result<(), Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(CONNECTION_STRING)
        .await
        .unwrap();

    let mut store = ThisQueryStore::new(pool);

    let id = uuid::Uuid::new_v4().to_string();

    // loading nonexisting query returns default constructor
    assert_eq!(
        store.load(id.as_str()).await.unwrap(),
        ThisQueryContext::new(id.to_string(), 0, Default::default())
    );

    let context = ThisQueryContext::new(
        id.to_string(),
        1,
        CustomerContactQuery {
            name: "".to_string(),
            email: "test@email.com".to_string(),
            latest_address: "one address".to_string(),
        },
    );

    store.commit(context).await.unwrap();

    let stored_context = store.load(&id).await.unwrap();

    assert_eq!(
        stored_context,
        ThisQueryContext::new(
            id.to_string(),
            1,
            CustomerContactQuery {
                name: "".to_string(),
                email: "test@email.com".to_string(),
                latest_address: "one address".to_string(),
            },
        )
    );

    Ok(())
}

#[test]
fn test_commit_and_load_queries() {
    tokio_test::block_on(commit_and_load_queries()).unwrap();
}
