## Building the application

For a bare minimum operating application we are missing a number of components including:

- a Restful API or other interface
- non-volatile persistence
- useful queries

A demo application with examples of all of these features is available in the
[cqrs-demo](https://github.com/serverlesstechnology/cqrs-demo) project.

The [persist module](https://docs.rs/cqrs-es/0.3.0/cqrs_es/persist/index.html) contains the generic entities 
needed for a backing event store. 
A database repository handles the implementation specifics with three options currently available:
- [PostgreSQL](https://www.postgresql.org/) -  [postgres-es](https://crates.io/crates/postgres-es)
- [MySQL](https://www.mysql.com/) - [mysql-es](https://crates.io/crates/mysql-es)
- [DynamoDb](https://aws.amazon.com/dynamodb/) - [dynamo-es](https://crates.io/crates/dynamo-es)

These libraries also provide persistence for simple queries. 
Note that `postgres-es` is used for examples in this user guide but all the crates have similar methods available.

For using `postgres-es` for persistence, add to the dependencies section of your `Cargo.toml`:

```toml
[dependencies]
cqrs-es = "0.3.0"
postgres-es = "0.3.0"
```