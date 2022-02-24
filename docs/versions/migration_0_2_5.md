## Migrating guide to v0.2.5

> v0.2.4 ==> v0.2.5

### Aggregate changes
The error payload returned from business logic is now configurable. 
The `UserErrorPayload` struct is still available as a reference implementation.

As part of the aggregate implementation two changes are needed.
- You must specify the error type, using the `UserErrorPayload` that originally was required will simplify this.
- The signature for `handle` now includes the custom error as part of the error result.
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
this is more complex with configurable user error payloads. 
If you continue to use the `UserErrorPayload` we have implemented `From<&str>` in order to 
provide the same functionality.

```rust
    // Formerly this was
    // let error = AggregateError::new("the expected error message");

    // Updated simple implementation
    let error: AggregateError<UserErrorPayload> = "the expected error message".into();
```

### Boxed causes and additional error enumerations

We've received a number of requests for help debugging applications in the early stages while configuration is still being tweaked.
To help with this we've added new error enumerations to better identify the root cause of unexpected errors.
- AggregateError::DatabaseConnectionError
- AggregateError::DeserializationError

We're also passing the root cause errors back (rather than just the error messages) to provide additional debugging 
information.

```rust
        let error_message = match &aggregate_error {
                AggregateError::UserError(e) => serde_json::to_string(e).unwrap(),
                AggregateError::TechnicalError(e) => e.to_string(),
                AggregateError::AggregateConflict => "command collision encountered, please try again".to_string(),
                
                // New error enumerations
                AggregateError::DatabaseConnectionError(e) => e.to_string(),
                AggregateError::DeserializationError(e) => e.to_string(),
        };
 ```
