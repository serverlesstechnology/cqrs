use postgres::{
    Client,
    NoTls,
};

use cqrs_es2_core::{
    example_impl::*,
    QueryContext,
};

use crate::{
    postgres_store::QueryStore,
    IQueryStore,
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

#[test]
fn commit_and_load_queries() {
    let conn = Client::connect(CONNECTION_STRING, NoTls).unwrap();
    let mut store = ThisQueryStore::new(conn);

    let id = uuid::Uuid::new_v4().to_string();

    // loading nonexisting query returns default constructor
    assert_eq!(
        store.load(id.as_str()).unwrap(),
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

    store.commit(context).unwrap();

    let stored_context = store.load(&id).unwrap();

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
}
