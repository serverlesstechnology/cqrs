# Change log

#### `v0.4.2`
- Add `append_query` method to the framework.

#### `v0.4.0`
- Modified the aggregate `handle` method to expect a reply of `Self::Error`. 
Previously `AggregateError` was returned which allowed overloading enum branches other than the `UserError`, this is no longer allowed.
- Added `Services` type to the aggregate trait. 
Any external services used should now be configured with the CqrsFramework rather than attached to a command.
- The TestFramework now provides the `expect_error` validator that is available if the `Self::Error` implements `PartialEq`.
- Removed deprecated `EventEnvelope` methods and `UserErrorPayload` struct from use in examples.

#### `v0.3.2`
- Removed deprecated methods and structs from use in examples.

#### `v0.3.1`
- Removed the `timestamp` field from the database tables. This field is not needed by the framework or repositories and its use in examples led to confusion.
- Deprecated UserErrorPayload, this will be removed in v0.4.0. User should create a custom error for their aggregate.
- Deprecated helper methods on `EventEnvelope`. These methods will be removed in v0.4.0 however the fields on `EventEnvelope` will remain public.

#### `v0.3.0`
> See the [v0.2.5 ==> v0.3.0 migration guide](migration_0_3_0.md) for more details.

- Published a new persistence repository, [dynamo-es](https://crates.io/crates/dynamo-es), providing an underlying persistence layer based on [AWS' DyanomoDb](https://aws.amazon.com/dynamodb/).
- The `handle` method within the `Aggregate` trait is now async. This will greatly simplify calling asynchronous clients and services from the aggregate logic.
- Deprecation of common peristence crate [persist-es](https://crates.io/crates/persist-es), all logic has moved to the `persist` module of [cqrs-es](https://crates.io/crates/cqrs-es).
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
