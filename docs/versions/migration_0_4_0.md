## Migrating guide to v0.4.0

> v0.3.x ==> v0.4.0

### The `Aggregate` trait now has a `Services` type that must be implemented
Services that were previously injected with the command can now be configured with the aggregate and injected by the `CqrsFramework`.
To use this feature your services must be thread-safe, i.e., they should implement `Send` & `Sync` (these are
[auto traits](https://doc.rust-lang.org/reference/special-types-and-traits.html#auto-traits) and are implemented
automatically byt the compiler if the logic is safe for use across threads).

### The `handle` method within the `Aggregate` trait has changed
Logic within the command handler can now use asynchronous clients and services directly. 

The signature for `handle` now includes a borrowed reference to the configured services. 
Additionally, the result now just returns the configured error, it is no longer necessary to wrap the error in an `AggregateError`.
```rust,ignore
    impl Aggregate for MyAggregate {
        Services = MyServices;
        ...
    
        async fn handle(&self, command: Self::Command, services: &Self::Services) -> Result<Vec<Self::Event>, Self::Error> {
            ...
        }
    }
```

Note that the result of the `execute` method on `CqrsFramework` will still return an `AggregateError` with the `UserError`
variant wrapping the returned error.

### Aggregate test fixtures

- The `then_expect_error` method on `AggregateTestExecutor` is available to directly test a resulting aggregate error.
*Note that the configured error must implement `PartialEq` to take advantage of this.*
- The `inspect_result` method will return the command handler result in order to directly test.
- Test fixtures must now be configured with a service stub when created.

```rust,ignore
// Formerly create with the `default` method.
// let test_framework: TestFramework<MyAggregate> = TestFramework<MyAggregate>::default();

// A service now must be injected to create the test harness.
let test_framework: TestFramework<MyAggregate> = TestFramework<MyAggregate>::with(MyService::mock());
```