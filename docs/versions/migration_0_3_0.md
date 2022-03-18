## Migrating guide to v0.3.0

> v0.2.5 ==> v0.3.0

### The `handle` method within the `Aggregate` trait is now async
Logic within the command handler can now use asynchronous clients and services directly. 

The signature for `handle` now includes the `async` keyword:
```rust
    impl Aggregate for TestAggregate {
        ...
    
        async fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, AggregateError<Self::Error>> {
            ...
        }
    }
```

### Deprecation of common peristence crate
The [persist-es crate](https://crates.io/crates/persist-es) used for housing logic that is common across the three
peristence crates has been deprecated. All components have been moved to the `persist` module of
[cqrs-es](https://crates.io/crates/cqrs-es).
This should only require a change to the namespace of any imports.

E.g.,
```rust
// Previous namespace 'persist_es' should now be 'cqrs_es::persist'
// use persist_es::{GenericQuery,ViewRepository};
use cqrs_es::persist::{GenericQuery,ViewRepository};
```

### Changes to persisted tables
Significant changes to event, snapshot and test view tables have been made.
See individual persistence packages for details of each.

### Aggregate test fixtures
A Tokio thread runner has been added to the test fixtures so these should not need any changes due to the change
in the Aggregate command handler.

The `then_expect_error` method on `AggregateTestExecutor` has been deprecated in order to be repurposed in v0.3.0, 
please use `then_expect_error_message` instead.

### String replaces `&'static str` in interfaces

The `Aggregate` and `DomainEvent` interfaces had methods producing `&'static str`, in all cases these should not return 
a String.