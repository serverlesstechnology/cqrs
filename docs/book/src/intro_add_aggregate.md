## Aggregate

With the command and event in place we can now start adding our business logic. 
In Domain Driven Design all of this logic belongs within the aggregate which
for our example we will call name `BankAccount`.

And for our simple set of business rules, we will use two fields.

```rust,ignore
#[derive(Serialize, Default, Deserialize)]
pub struct BankAccount {
    opened: bool,
    // this is a floating point for our example, don't do this IRL
    balance: f64,
}
```

In order to operate within the `cqrs-es` framework, we will need the traits, `Default`, `Serialize` and `Deserialize`
(all usually derived) and we will implement `cqrs_es::Aggregate`, minus any of the business logic. 

```rust
#[async_trait]
impl Aggregate for BankAccount {
    type Command = BankAccountCommand;
    type Event = BankAccountEvent;
    type Error = BankAccountError;
    type Services = BankAccountServices;

    // This identifier should be unique to the system.
    fn aggregate_type() -> String {
        "Account".to_string()
    }

    // The aggregate logic goes here. Note that this will be the _bulk_ of a CQRS system
    // so expect to use helper functions elsewhere to keep the code clean.
    async fn handle(&self, command: Self::Command, services: Self::Services) -> Result<Vec<Self::Event>, Self::Error> {
        todo!()
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            BankAccountEvent::AccountOpened { .. } => {
                self.opened = true
            }

            BankAccountEvent::CustomerDepositedMoney { amount: _, balance } => {
                self.balance = balance;
            }

            BankAccountEvent::CustomerWithdrewCash { amount: _, balance } => {
                self.balance = balance;
            }

            BankAccountEvent::CustomerWroteCheck {
                check_number: _,
                amount: _,
                balance,
            } => {
                self.balance = balance;
            }
        }

    }
}

```

### Identifying the aggregate when persisted

The `aggregate_type` method is used by the cqrs-es framework to uniquely identify the aggregate and event
when serialized for persistence. Each aggregate should use a unique value within your application.
```rust
    fn aggregate_type() -> String {
        "Account".to_string()
    }
```

### Handling commands

The `handle` method of this trait is where _all_ of the business logic will go, for now we will leave that out and just return an empty vector.

```rust
    // note that the aggregate is immutable and an error can be returned
    async fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, AggregateError<Self::Error>> {
        todo!()
    }
```
The `handle` method does not allow any mutation of the aggregate, state should be changed _only_ by emitting events.

### Applying committed events

Once events have been committed they will need to be applied to the aggregate in order for it to update its state.
```rust
    // note the aggregate is mutable and there is no return type
    fn apply(&mut self, event: Self::Event) {
        match event {
            BankAccountEvent::AccountOpened { .. } => {
                self.opened = true
            }
            
            BankAccountEvent::CustomerDepositedMoney { amount: _, balance } => {
                self.balance = balance;
            }
            
            BankAccountEvent::CustomerWithdrewCash { amount: _, balance } => {
                self.balance = balance;
            }
            
            BankAccountEvent::CustomerWroteCheck {
                check_number: _,
                amount: _,
                balance,
            } => {
                self.balance = balance;
            }
        }
    }
```
Note that the `apply` function has no return value. The act of applying an event is simply bookkeeping, the action has
already taken place.

> An event is a historical fact, it can be ignored, but it should never cause an error.