use cqrs_es2_core::{
    example_impl::*,
    Error,
    QueryContext,
};

use crate::{
    memory_store::QueryStore,
    IQueryStore,
};

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

async fn check_memory_query_store() -> Result<(), Error> {
    let mut store = ThisQueryStore::default();

    let id = "test_id_A";

    let stored_context = store.load(&id).await.unwrap();

    assert_eq!(
        stored_context,
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
fn test_memory_query_store() {
    tokio_test::block_on(check_memory_query_store()).unwrap();
}
