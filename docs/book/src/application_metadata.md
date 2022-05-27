## Including metadata with our commands

Any useful application will require much more information than what is solely needed to satisfy the domain (business) logic.
This additional data could be needed for debugging, security, an audit trail or a variety of reasons. 
Some examples include:
- server name, region or other operational information
- username that authorized the request
- IP address that made the call
- date and time that the command was processed
- a request id for distributed tracing

A Domain Event is intended to only carry information that is pertinent to the domain logic, 
this additional information should be added as metadata when the command is processed.
All events that are produced will be persisted along with a copy of this metadata.
Any configured Queries will also receive the metadata along with the event payload as part of an `EventEnvelope`.

The `CqrsFramework` expects the metadata in the form of key-value pairs stored in a standard `HashMap<String,String>`, 
this metadata should be passed along with the command at the time of execution. 

```rust
async fn process_command(
    cqrs: PostgresCqrs<BankAccount>,
    command: BankAccountCommand,
) -> Result<(), AggregateError<BankAccountError>> {
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), chrono::Utc::now().to_rfc3339());

    cqrs.execute_with_metadata("agg-id-F39A0C", command, metadata).await
}
```
