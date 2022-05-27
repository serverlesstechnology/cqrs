## Domain Events

Next we will need to create some domain events. Note that we qualify events with 'domain' to differentiate them from
other events that might exist within our application. These are domain events because they make assertions about
changes in the aggregate state. 

In the `cqrs-es` framework the domain events are expected to be an enum with payloads similar to the commands,
this will give us a single root event for each aggregate. 

The enum as well as the payloads should derive several traits.

- `Debug` - used for error handling and testing.
- `Clone` - the event may be passed to a number of downstream queries in an asynchronous manner and will need to be cloned.
- `Serialize, Deserialize` - serialization is essential for both storage and publishing to distributed queries.
- `PartialEq` - we will be adding a lot of tests to verify that our business logic is correct.

### Adding events and payloads

Let's add four self-descriptive events as part of a single enum.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BankAccountEvent {
    AccountOpened {
        account_id: String,
    },
    CustomerDepositedMoney {
        amount: f64,
        balance: f64,
    },
    CustomerWithdrewCash {
        amount: f64,
        balance: f64,
    },
    CustomerWroteCheck {
        check_number: String,
        amount: f64,
        balance: f64,
    },
}
```

Again, all of our events are named in the past tense,
[this is important](https://martinfowler.com/bliki/UbiquitousLanguage.html).

Our events now need to implement `cqrs_es::DomainEvent<BankAccount>` to provide an `event_name` and `event_version`
for each event.
This will be important later in any production system when events need to be changed 
(see [event upcasters](event_upcasters.md)).

```rust
impl DomainEvent for BankAccountEvent {
    fn event_type(&self) -> String {
        let event_type: &str = match self {
            BankAccountEvent::AccountOpened { .. } => "AccountOpened",
            BankAccountEvent::CustomerDepositedMoney { .. } => "CustomerDepositedMoney",
            BankAccountEvent::CustomerWithdrewCash { .. } => "CustomerWithdrewCash",
            BankAccountEvent::CustomerWroteCheck { .. } => "CustomerWroteCheck",
        };
        event_type.to_string()
    }
    
    fn event_version(&self) -> String {
        "1.0".to_string()
    }
}
```