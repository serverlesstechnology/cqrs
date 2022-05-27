## Event upcasters

Over time the domain model will need to be modified to adapt to new business rules,
and with event sourcing the domain model directly relates to events. 
Event changes can be minimized by keeping events small and focused, but they will be needed.
This can be a challenge because domain events are append-only and immutable.

As an example, if our bank services only local customers there is no need to identify the state as part of their address,
this is understood. The payload for an `UpdateAddress` event might look something like:
```json
{
  "UpdateAddress": {
    "address": "912 Spring St",
    "city": "Seattle"
  }
}
```

If however the bank begins servicing customers in other states we'll need additional information in our payload, e.g.,
```json
{
  "UpdateAddress": {
    "address": "912 Spring St",
    "city": "Seattle",
    "state": "WA"
  }
}
```

We are event sourced, so we will need to load past events in order to build our aggregate to process new commands.
However, the persisted form of the event no longer matches the new structure. 

The naive solution of versioning events is not preferred due to the duplication of both business logic and tests.
This duplication requires additional maintenance, a risk of logic diverging for the same tasks, and leaves a burden
on the developer of any new code to understand the legacy changes.

The preferred solution is to use upcasters to convert a legacy event payload to the structure that is expected by the
current aggregate logic.

### Event Upcaster

The `EventUpcaster` trait provides the functionality to make this conversion.
A persistence repository will use any configured upcasters to 'upcast' events as they are loaded.
For each event, the stored `eveent_type` and `event_version` will be compared to each upcaster to determine if it 
should be upcast, and the to upcast it if needed.
```rust
pub trait EventUpcaster: Send + Sync {
    fn can_upcast(&self, event_type: &str, event_version: &str) -> bool;
    fn upcast(&self, event: SerializedEvent) -> SerializedEvent;
}
```
The `EventUpcaster` trait provides flexibility to modify a serialized event in any way needed including changing the
name and modifying metadata. 
In most cases this flexibility is not needed and a `SemanticVersionEventUpcaster` can be used, this implementation
will use the provided function to modify the event payload of any event with a matching `event_name` and with an
`event_version` that is previous to the configured value.

For the above example we only need to add a field:
```rust,ignore
let upcaster = SemanticVersionEventUpcaster::new("UpdateAddress", "0.3.0", Box::new(my_upcaster));

fn my_upcast_fn(mut payload: Value) -> Value {
    match payload.get_mut("UpdateAddress").unwrap() {
        Value::Object(object) => {
            object.insert("state".to_string(), Value::String("WA".to_string()));
            payload
        }
        _ => panic!("invalid payload encountered"),
    }
}
```
