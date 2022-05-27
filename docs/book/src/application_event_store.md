## Persisted Event Store

A `PersistedEventStore` is used to back the CqrsFramework and handle the storing and loading of aggregates 
(including domain events) in a database. 
The `PersistedEventStore` relies on a `PersistedEventRepository` for the actual database access of events and snapshots.
For the `postgres-es` crate this is implemented by a `PostgresEventRepository` which in turn relies on a
database connection pool.

Creating a `PostgresEventRepository`
```rust
fn configure_repo() -> PostgresEventRepository {
    let connection_string = "postgresql://test_user:test_pass@localhost:5432/test";
    let pool: Pool<Postgres> = default_postgress_pool(connection_string).await;
    PostgresEventRepository::new(pool)
}
```

The default repository will expect to find tables named `events` and `snapshots`, but the table names are configurable.
To create these tables in a PostgreSql database (see database initialization files for other repository crates):
```sql
CREATE TABLE events
(
    aggregate_type text                         NOT NULL,
    aggregate_id   text                         NOT NULL,
    sequence       bigint CHECK (sequence >= 0) NOT NULL,
    event_type     text                         NOT NULL,
    event_version  text                         NOT NULL,
    payload        json                         NOT NULL,
    metadata       json                         NOT NULL,
    timestamp      timestamp with time zone DEFAULT (CURRENT_TIMESTAMP),
    PRIMARY KEY (aggregate_type, aggregate_id, sequence)
);

CREATE TABLE snapshots
(
    aggregate_type   text                                 NOT NULL,
    aggregate_id     text                                 NOT NULL,
    last_sequence    bigint CHECK (last_sequence >= 0)    NOT NULL,
    current_snapshot bigint CHECK (current_snapshot >= 0) NOT NULL,
    payload          json                                 NOT NULL,
    timestamp        timestamp with time zone DEFAULT (CURRENT_TIMESTAMP),
    PRIMARY KEY (aggregate_type, aggregate_id, last_sequence)
);
```
Note that the `snapshots` table is not needed for pure event sourcing.
