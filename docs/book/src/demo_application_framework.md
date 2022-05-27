## Putting everything together

The final piece of our test application is a CQRS framework to load up the aggregate, process incoming commands,
persist the events and apply them to our queries. This is provided by a `CqrsFramework` component which takes an
`EventStore` and a vector of boxed `Query`s. 

Wiring this all up and firing two commands:

```rust
#[tokio::test]
async fn test_event_store() {
    let event_store = MemStore::<BankAccount>::default();
    let query = SimpleLoggingQuery {};
    let cqrs = CqrsFramework::new(event_store, vec![Box::new(query)]);

    let aggregate_id = "aggregate-instance-A";

    // deposit $1000
    cqrs.execute(aggregate_id, DepositMoney{
        amount: 1000_f64
    }).unwrap();

    // write a check for $236.15
    cqrs.execute(aggregate_id, WriteCheck{
        check_number: "1337".to_string(),
        amount: 236.15
    }).unwrap();
}
```

To run the test we should ensure that rust does not consume our output.

```
cargo test -- --nocapture
```

Which should give us output something like this:

```
running 1 test
loading: 0 events for aggregate ID 'aggregate-instance-A'
storing: 1 new events for aggregate ID 'aggregate-instance-A'
aggregate-instance-A-1
{
  "CustomerDepositedMoney": {
    "amount": 1000.0,
    "balance": 1000.0
  }
}
loading: 1 events for aggregate ID 'aggregate-instance-A'
storing: 1 new events for aggregate ID 'aggregate-instance-A'
aggregate-instance-A-2
{
  "CustomerWroteCheck": {
    "check_number": "1137",
    "amount": 236.15,
    "balance": 763.85
  }
}
```

Here we see the output from our `SimpleLoggingQuery` along with some logging from the `MemStore` which is just what we hoped 
for.

This shows our entire framework working including loading events, rebuilding the aggregate, processing commands and
distributing events to a query. Next, we will move on to actually using this in an application.