## Migrating guide to v0.2.5

> v0.2.4 ==> v0.2.5

### Aggregate changes
The error payload returned from business logic is now configurable. 
The `UserErrorPayload` struct is still available as a reference implementation.

As part of the aggregate implementation two changes are needed.
- You must specify the error type, using the `UserErrorPayload` that originally was required will simplify this.
- The signagure for `handle` now includes the custom error as part of the error result.
```rust
    impl Aggregate for TestAggregate {
        type Error = UserErrorPayload;
    
        ...
    
        fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, AggregateError<Self::Error>> {
            ...
        }
    }
```

### Simple creation error

Previously helper functions were available to create a simple aggregate error from a `&str`,
this is more complex with configurable user error payloads. If you continue to use the `UserErrorPayload`
this will remain available but has been renamed to `new_user_error`.

```rust
    // Formerly this was
    // let error = AggregateError::new("the expected error message");

    // Updated simple implementation
    let error = AggregateError::new_user_error("the expected error message");
```
