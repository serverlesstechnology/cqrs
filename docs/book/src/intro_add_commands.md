## Commands

In order to make changes to our system we will need commands. 
These are the simplest components of any CQRS system and consist of little more than packaged data.

When designing commands an easy mental model to use is that of an HTTP API.
Each virtual endpoint would receive just the data that is needed to operate that function. 

```rust,ignore
#[derive(Debug, Deserialize)]
pub enum BankAccountCommand {
    OpenAccount { account_id: String },
    DepositMoney { amount: f64 },
    WithdrawMoney { amount: f64 },
    WriteCheck { check_number: String, amount: f64 },
}
```

Note that the `Deserialize` trait is derived. 
This is not yet needed, but it will be useful when building out a full application.
The most common way to receive commands from a user is via an HTTP body that can be directly deserialized.