use cqrs_es2::{
    postgres_store::{
        EventStore,
        QueryStore,
    },
    Error,
    Repository,
};

use crate::cqrs::db_connection;

use super::super::{
    aggregate::BankAccount,
    commands::BankAccountCommand,
    dispatchers::LoggingDispatcher,
    events::BankAccountEvent,
    queries::BankAccountQuery,
};

type ThisEventStore =
    EventStore<BankAccountCommand, BankAccountEvent, BankAccount>;

type ThisQueryStore = QueryStore<
    BankAccountCommand,
    BankAccountEvent,
    BankAccount,
    BankAccountQuery,
>;

type ThisRepository = Repository<
    BankAccountCommand,
    BankAccountEvent,
    BankAccount,
    ThisEventStore,
>;

pub async fn get_event_store() -> Result<ThisRepository, Error> {
    Ok(ThisRepository::new(
        ThisEventStore::new(db_connection().await.unwrap(), true),
        vec![
            Box::new(get_query_store().await.unwrap()),
            Box::new(LoggingDispatcher::new()),
        ],
    ))
}

pub async fn get_query_store() -> Result<ThisQueryStore, Error> {
    Ok(ThisQueryStore::new(
        db_connection().await.unwrap(),
    ))
}
