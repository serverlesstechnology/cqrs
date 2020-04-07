# CQRS

A lightweight, opinionated CQRS and event sourcing framework targeting serverless architectures.

![Build tag](https://codebuild.us-west-2.amazonaws.com/badges?uuid=eyJlbmNyeXB0ZWREYXRhIjoia3ZYcXozMjVZaFhoTldlUmhHemlWVm9LUjVaTC9LN3dSTFZpMkVTTmRycElkcGhJT3g2TUdtajZyRWZMd01xNktvUkNwLzdZYW15bzJkZldQMjJWZ1dNPSIsIml2UGFyYW1ldGVyU3BlYyI6InFORDNyaFFEQUNFQkE1NlUiLCJtYXRlcmlhbFNldFNlcmlhbCI6MX0%3D&branch=master)

### Installation

    [dependencies]
    cqrs-es = "0.0.2"
    
### Opinions

- Aggregate persistence is via event sourcing only.
- Metadata is implemented only as a `HashMap<String,String>`. 
Further, the `MetadataSupplier` that the user provides has no insight into the event or aggregate that 
it supplies metadata for. This may be changed.
- JSON serialization only.
- Generics are preferred over boxed traits.
- Persistence is implemented through a Postgres database.

### Todos/research

- Event upcasters.
- Some additional framework around `GenericViewRepository` to simplify event replay.
- Explore options for increasing the usefulness of `MetadataSupplier`.
- Event serialization uses the event type as the root node of the JSON tree. This simplifies
deserialization but is non-standard.
- Persistence implementation for DynamoDb.