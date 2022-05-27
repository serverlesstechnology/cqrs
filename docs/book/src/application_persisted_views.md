## Queries with persisted views

A `ViewRepository` provides a simple database backed repository for views that do not require multiple indexes.
This is designed to back work with a `GenericQuery` to apply events to a view synchronously immediately after those 
events are committed.

A `GenericQuery` will load the view, apply any events, and store the updated version back in the database.
The logic for the update is placed in a `View` implementation. 
For our bank account example this might look like:
```rust
impl View<BankAccount> for BankAccountView {
    fn update(&mut self, event: &EventEnvelope<BankAccount>) {
        match &event.payload {
            BankAccountEvent::CustomerDepositedMoney { amount, balance } => {
                self.ledger.push(LedgerEntry::new("deposit", *amount));
                self.balance = *balance;
            }
            ...
        }
    }
}
```

The view repositories use the same database connection as the event repositories, for `postgres-es` this is a database
connection pool. 
```rust
type MyViewRepository = PostgresViewRepository<MyView,MyAggregate>;

fn configure_view_repository(db_pool: Pool<Postgres>) -> MyViewRepository {
    PostgresViewRepository::new("my_view_name", db_pool)
}
```

The database must have a table prepared before use, where the table name should match the value passed while
initiating the `PostgresViewRepository`.
```sql
CREATE TABLE my_view_name
(
    view_id text                        NOT NULL,
    version bigint CHECK (version >= 0) NOT NULL,
    payload json                        NOT NULL,
    PRIMARY KEY (view_id)
);
```

To use this view repository with a `GenericQuery` it must be configured with the CqrsFramework.

```rust
fn configure_cqrs(store: PostgresEventStore, my_view_repo: MyViewRepository) -> CqrsFramework {
    let my_query = GenericQuery::<MyViewRepository, MyView, MyAggregate>::new(my_view_repo);
    let my_query = Box::new(my_query);
    CqrsFramework::new(store, vec![my_query]);
}
```
