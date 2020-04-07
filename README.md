# CQRS

A lightweight, opinionated CQRS and event sourcing framework targeting serverless architectures.

### Opinions

1. Aggregate persistence is via event sourcing only.
1. Snapshots are not supported (protip: if you use snapshots, you're not event sourced, see opinion 1).
1. Metadata is implemented only as a `HashMap<String,String>`. 
1. Further, the `MetadataSupplier` that the user provides has no insight into the event or aggregate that 
it supplies metadata for. This may be changed
1. JSON serialization
