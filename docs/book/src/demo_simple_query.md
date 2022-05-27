## A simple query

The command processing portion of a CQRS handles updates to the system but provides no insight into the current
state. For this we will need one or more queries that read the events as they are committed. In the `cqrs-es` crate
these events should implement the `Query` trait.

For our first query, we will just print the aggregate instance ID, sequence number and the event payload.

```rust
struct SimpleLoggingQuery {}

#[async_trait]
impl Query<BankAccount> for SimpleLoggingQuery {
    async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<BankAccount>]) {
        for event in events {
            println!("{}-{}\n{:#?}", aggregate_id, event.sequence, &event.payload);
        }
    }
}
```

Note that the trait's sole method takes a vector of 
[`EventEnvelope`](https://docs.rs/cqrs-es/0.3.0/cqrs_es/struct.EventEnvelope.html)s, 
a struct that contains the event along with supporting context and 
[metadata](application_metadata.md).
This allows queries to have the full context surrounding the event, important since a query may be 
interested in a very different set of fields than those of interest within the aggregate.

E.g., the user's IP address is likely unimportant for the business rules but could be of interest in a query 
used for security audits
