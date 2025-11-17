## Migrating guide to v0.5.0

> v0.4.x ==> v0.5.0

### Significant changes to the `Aggregate::handle` interface and operation
Several changes were made to the aggregate handler interface. These are:
- The method now requires that the aggregate be mutable.
- Events are now longer returned from the method.
- An `EventSink` struct is now used to write events (previously returned from the method).

```rust
async fn handle(&self, command: Self::Command, service: &Self::Services) -> Result<Vec<Self::Event>, Self::Error>
```

```rust
async fn handle(&mut self, command: Self::Command, service: &Self::Services, sink: &EventSink<Self>) -> Result<(), Self::Error>
```

### Update command handler logic
Previously, writing an event was handled by returning it, within a vector, as the `handle` result.
This logic should be modified to write the event to the provided `EventSink`.
Additionally, a successful command should return an empty `Ok(())`.

Within the command handler logic, what was previously
```
let result: Vec<CustomerEvent> = Default::default();
result.append(CustomerEvent::CustomerAdded);

...

Ok(result)
```
or
```
return Ok(vec![CustomerEvent::CustomerAdded]);
```
Should be converted to:
```
sink.write(CustomerEvent::CustomerAdded, self)

...

Ok(())
```
