use cqrs_es2::{
    postgres_store::{
        EventStore,
        QueryStore,
    },
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

pub fn get_event_store() -> ThisRepository {
    ThisRepository::new(
        ThisEventStore::new(db_connection(), true),
        vec![
            Box::new(get_query_store()),
            Box::new(LoggingDispatcher::new()),
        ],
    )
}

pub fn get_query_store() -> ThisQueryStore {
    ThisQueryStore::new(db_connection())
}
