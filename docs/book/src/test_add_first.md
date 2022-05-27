## Adding aggregate tests

Now that we have the basic components in place we can begin setting up our aggregate tests. These are the tests that
we will use to verify the business logic for our application. Testing is one of the most valuable aspects of CQRS/event 
sourcing as it allows us to configure tests that have no coupling with our application logic.

We can do this because we rely only on events for past state, so no amount of refactoring of our application logic will
affect the whether a test passes or fails (as long as the result of the command is the same).
These tests follow a pattern that you are likely familiar with:

- Given some past events
- When a command is applied
- Then some result is expected

Let's first add a test module and define a new `AccountTestFramework` convenience type for our test framework.

```rust
#[cfg(test)]
mod aggregate_tests {
    use super::*;
    use cqrs_es::test::TestFramework;

    type AccountTestFramework = TestFramework<BankAccount>;
}
```

### A first aggregate test

Now within our `aggregate_tests` module we will add our first test. 
We do not require any previous events so we can initiate our test with the `given_no_previous_events` method.
Let's fire a `DepositMoney` command and expect to a `CustomerDepositedMoney` event. 

```rust
#[test]
fn test_deposit_money() {
    let expected = BankAccountEvent::CustomerDepositedMoney { amount: 200.0, balance: 200.0 };

    AccountTestFramework::default()
        .given_no_previous_events()
        .when(DepositMoney{ amount: 200.0 })
        .then_expect_events(vec![expected]);
}
```

Now if we run this test, we should see a test failure with output that looks something like this:

```
thread 'aggregate_tests::test' panicked at 'assertion failed: `(left == right)`
  left: `[]`,
 right: `[CustomerDepositedMoney{ amount: 200.0, balance: 200.0 }]`', <::std::macros::panic ...
```
We have not added any logic yet, so this is what we should see. 
We have told the test to expect a `CustomerDepositedMoney` event, but none has been produced.

### Adding business logic

Let's go back to our `Command` implementation for `DepositMoney` and fix this.

```rust
async fn handle(&self, command: Self::Command) -> Result<Vec<Self::Event>, AggregateError<Self::Error>> {
    match command {
        BankAccountCommand::DepositMoney { amount } => {
            let balance = self.balance + amount;
            Ok(vec![BankAccountEvent::CustomerDepositedMoney {
                amount,
                balance,
            }])
        }
        _ => Ok(vec![])
    }
}

```

And running our first test again - success!

### Dealing with previous events

Now we should verify that our logic is valid if there is a previous balance. For this, we will use the `given` method to 
initiate the test, along with a vector containing a sole previous event:

```rust
#[test]
fn test_deposit_money_with_balance() {
    let previous = BankAccountEvent::CustomerDepositedMoney { amount: 200.0, balance: 200.0 };
    let expected = BankAccountEvent::CustomerDepositedMoney { amount: 200.0, balance: 400.0 };

    AccountTestFramework::default()
        .given(vec![previous])
        .when(DepositMoney{ amount: 200.0 })
        .then_expect_events(vec![expected]);
}
```

These exercises feel a little-brain dead, but they provide a good example of how these tests are structured. 
Next we will start adding some real logic.