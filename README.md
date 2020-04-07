# CQRS

A lightweight, opinionated CQRS and event sourcing framework targeting serverless architectures.

### Opinions

- Aggregate persistence is via event sourcing only.
- Metadata is implemented only as a `HashMap<String,String>`. 
Further, the `MetadataSupplier` that the user provides has no insight into the event or aggregate that 
it supplies metadata for. This may be changed.
- JSON serialization only.
- Generics are preferred over boxed traits.

### Todos/research

- Event upcasters.
- Some additional framework around `GenericViewRepository` to simplify event replay.
- Explore options for increasing the usefulness of `MetadataSupplier`.
- Event serialization uses the event type as the root node of the JSON tree. This simplifies
deserialization but is non-standard.
- In large