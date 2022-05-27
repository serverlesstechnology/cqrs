## Adding more logic

In our simple example a customer can always deposit money, but making a cash withdrawal is another thing. We should ensure that
the customer has the requested funds available before releasing them, lest the account overdraw.

When discussing events, we noted that the process of applying events cannot produce an error since it is a past 
event. Instead, errors should be produced before the event is generated, during the processing of the command.

### Account withdrawal - happy path

First, let's add a test for a happy path withdrawal, again with a previous deposit using the `given`
initial method:

```rust
#[test]
fn test_withdraw_money() {
    let previous = BankAccountEvent::CustomerDepositedMoney { amount: 200.0, balance: 200.0 };
    let expected = BankAccountEvent::CustomerWithdrewCash { amount: 100.0, balance: 100.0 };

    AccountTestFramework::default()
        .given(vec![previous])
        .when(WithdrawMoney{ amount: 100.0 })
        .then_expect_events(vec![expected]);
}
```

Since we have not added any withdrawal logic yet this should fail. 
Let's correct this with some naive logic to produce the event:

```rust
    async fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, AggregateError<Self::Error>> {
        match command {
            BankAccountCommand::WithdrawMoney { amount } => {
                let balance = self.balance - amount;
                Ok(vec![BankAccountEvent::CustomerWithdrewCash {
                    amount,
                    balance,
                }])
            }
            ...
        }
    }
```

### Verify funds are available

Now we have success with our happy path test, but then there is nothing to stop a customer from withdrawing more than 
is deposited. 
Let's add a test case using the `then_expect_error` expect case:

```rust
#[test]
fn test_withdraw_money_funds_not_available() {
    let error = AccountTestFramework::default()
        .given_no_previous_events()
        .when(BankAccountCommand::WithdrawMoney { amount: 200.0 })
        .then_expect_error();
    assert_eq!("funds not available", &error.0)
}
```

We should see our new test fail since our naive logic cannot handle this yet.
Now we update our command logic to return an error when this situation arises:

```rust
    async fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, AggregateError<Self::Error>> {
        match command {
            BankAccountCommand::WithdrawMoney { amount } => {
                let balance = self.balance - amount;
                if balance < 0_f64 {
                    return Err(AggregateError::new("funds not available"));
                }
                Ok(vec![BankAccountEvent::CustomerWithdrewCash {
                    amount,
                    balance,
                }])
            }
            ...
        }
    }
```

And we should now see our test pass.

Note that handling a command is always an atomic process, either all produced events become a part of the factual 
history of this aggregate instance, or an error is returned.

