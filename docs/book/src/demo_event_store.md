## Using an event store

In an event sourced application the domain events are our source of truth, and to provide persistence we need an event
store. Any persistence mechanism can be used but there are a few things that we need from a production event store:

- append only
- load all events, in order of commit, for a single aggregate instance
- a guarantee that no events are missing
- optimistic locking on the aggregate instance
- provide additional metadata, outside of the event payload, for auditing or logging use

To provide our needed guarantees we identify any domain event by the combination of the aggregate type, the 
aggregate instance ID and the sequence number.
This allows us to correctly order all events, guarantee that we are not missing any events for an aggregate instance,
and to provide optimistic locking on append. 

To keep all of the context surrounding and event together with the event payload, we use an `EventEnvelope` consisting of:

- aggregate instance ID
- sequence number
- aggregate type
- payload
- metadata

### The `EventStore` trait

In our application we need an implementation of `EventStore` for appending and loading events.
For our test application we will use `MemStore`, the in-memory event store that ships with the `cqrs-es` crate.

```rust
use cqrs_es::mem_store::MemStore;

let event_store = MemStore::<BankAccount>::default();
```

This implementation will not give us any real persistence but it will allow us to get started with testing our
application. Later we will use another crate to provide a production capable implementation.