# Change log

#### `v0.2.6`
- Published a beta edition of [dynamo-es](https://crates.io/crates/dynamo-es) crate providing an underlying persistence layer based on [AWS' DyanomoDb](https://aws.amazon.com/dynamodb/).
- The `handle` method within the `Aggregate` trait is now async. This will greatly simplify calling asynchronous clients and services from the aggregate logic.
- The event and snapshot table names are now configurable in the persistence packages.
- Corrected a bug that caused [mysql-es](https://crates.io/crates/mysql-es) to return the wrong error when an optimistic lock violation was encountered.
- In `AggregateTestExecutor` the method `then_expect_error_message` was added to replace the now deprecated `then_expect_error`.

#### `v0.2.5`
> See the [v0.2.4 ==> v0.2.5 migration guide](migration_0_2_5.md) for more details.

- The payload for user errors in the aggregate is now configurable.
- Additional enumerations for `AggregateError`.
- Unexpected errors now return with the root cause rather than just the message.

#### `v0.2.4`
- Move to Rust 2021 edition.
- Audit and update dependencies.

#### `v0.2.3`
- Added upcasters to event stores.

#### `v0.2.2`
- Consolidated repositories to a single trait encompassing all functionality.

#### `v0.2.1`
- Moved generic persistence logic in from postgres-es package.
- Added event context information to event envelope.

#### `v0.2.0`
Moved to async/await for better tool support.

#### `v0.1.3`
Aggregates now consume events on `apply`.

#### `v0.1.2`
Require `Send + Sync` for queries.

#### `v0.1.1`
Require `Send + Sync` for support of multi-threaded applications.

#### `v0.1.0`
Corrected to move all command and event logic into the aggregate.
