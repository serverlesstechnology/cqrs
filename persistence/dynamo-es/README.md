# dynamo-es

> A DynamoDB implementation of the `PersistedEventRepository` trait in cqrs-es.

## Usage
Add to your Cargo.toml file:

```toml
[dependencies]
cqrs-es = "0.4.11"
dynamo-es = "0.4.11"
```

Requires access to a Dynamo DB with existing tables. See:
- [Sample database configuration](db/dynamo_db.yaml)
- [Sample database table layout](db/create_tables.sh)
- Use `docker-compose` and the `./db/create_tables.sh` script to quickly setup [a local database](docker-compose.yml)

### DynamoDb caveats
AWS' DynamoDb is fast, flexible and highly available, but it does 
[set some limitations](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ServiceQuotas.html)
that must be considered in the design of your application.

#### Maximum limit of 25 operations in any transaction

Events are inserted in a single transaction, which limits the number of events that can be handled from a single command
using this repository. To operate correctly a command must not produce more than
- 25 events if using [an event store without snapshots](https://docs.rs/cqrs-es/latest/cqrs_es/persist/struct.PersistedEventStore.html#method.new_event_store)
- 24 events if using [snapshots](https://docs.rs/cqrs-es/latest/cqrs_es/persist/struct.PersistedEventStore.html#method.new_snapshot_store)
or [an aggregate store](https://docs.rs/cqrs-es/latest/cqrs_es/persist/struct.PersistedEventStore.html#method.new_aggregate_store)
 
#### Item size limit of 400 KB
A single event should never reach this size, but a large serialized aggregate might.
If this is the case for your aggregate beware of using [snapshots](https://docs.rs/cqrs-es/latest/cqrs_es/persist/struct.PersistedEventStore.html#method.new_snapshot_store)
or [an aggregate store](https://docs.rs/cqrs-es/latest/cqrs_es/persist/struct.PersistedEventStore.html#method.new_aggregate_store).

#### Maximum request size of 1 MB
This could have the same ramifications as the above for [snapshots](https://docs.rs/cqrs-es/latest/cqrs_es/persist/struct.PersistedEventStore.html#method.new_snapshot_store)
or [an aggregate store](https://docs.rs/cqrs-es/latest/cqrs_es/persist/struct.PersistedEventStore.html#method.new_aggregate_store).
Additionally, an aggregate instance with a large number of events may reach this threshold. 
To prevent an error while loading or replaying events, 
[set the streaming channel size](https://docs.rs/dynamo-es/latest/dynamo_es/struct.DynamoEventRepository.html#method.with_streaming_channel_size)
to a value that ensures you won't exceed this threshold.


### Testing

Requires access to DynamoDb with existing tables. This can be created locally using the included 
`docker-compose.yml` and database initialization script.

To prepare a local test environment (requires a local installation of 
[Docker](https://www.docker.com/products/docker-desktop) and 
[AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-welcome.html)):
```
docker-compose up -d
./db/create_tables.sh
```

It is recommended that tables are configured to allow only transactions.
See:
https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/transaction-apis-iam.html

---

Things that could be helpful:
- [User guide](https://doc.rust-cqrs.org) along with an introduction to CQRS and event sourcing.
- [Demo application](https://github.com/serverlesstechnology/cqrs-demo) using the warp http server.
- [Change log](https://github.com/serverlesstechnology/cqrs/blob/main/change_log.md)

[![Crates.io](https://img.shields.io/crates/v/dynamo-es)](https://crates.io/crates/dynamo-es)
[![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/dynamo-es)
![build status](https://codebuild.us-west-2.amazonaws.com/badges?uuid=eyJlbmNyeXB0ZWREYXRhIjoiVVUyR0tRbTZmejFBYURoTHdpR3FnSUFqKzFVZE9JNW5haDZhcUFlY2xtREhtaVVJMWsxcWZOeC8zSUR0UWhpaWZMa0ZQSHlEYjg0N2FoU2lwV1FsTXFRPSIsIml2UGFyYW1ldGVyU3BlYyI6IldjUVMzVEpKN1V3aWxXWGUiLCJtYXRlcmlhbFNldFNlcmlhbCI6MX0%3D&branch=main)
