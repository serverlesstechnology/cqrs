use crate::config::cqrs_framework;
use crate::domain::aggregate::BankAccount;
use crate::queries::BankAccountView;
use postgres_es::{default_postgress_pool, PostgresCqrs, PostgresViewRepository};
use std::sync::Arc;

#[derive(Clone)]
pub struct ApplicationState {
    pub cqrs: Arc<PostgresCqrs<BankAccount>>,
    pub account_query: Arc<PostgresViewRepository<BankAccountView, BankAccount>>,
}

pub async fn new_application_state() -> ApplicationState {
    // Configure the CQRS framework, backed by a Postgres database, along with two queries:
    // - a simply-query prints events to stdout as they are published
    // - `account_query` stores the current state of the account in a ViewRepository that we can access
    //
    // The needed database tables are automatically configured with `docker-compose up -d`,
    // see init file at `/db/init.sql` for more.
    let pool = default_postgress_pool("postgresql://demo_user:demo_pass@localhost:5432/demo").await;
    let (cqrs, account_query) = cqrs_framework(pool);
    ApplicationState {
        cqrs,
        account_query,
    }
}
