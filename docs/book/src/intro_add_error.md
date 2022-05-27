## Custom error and service

Aside from our domain data objects, we'll need two additional components to complete an aggregate.
An error to indicate a violation of the business rules, and a set of services that will be made available during command processing.

### User error

The `Aggregate` trait can return an error` from its `handle` method indicating that some rule of the business logic was violated,
this information will usually be returned to the user as an error message.
For example, an attempt to withdraw more money from a bank account than the current balance would return this error
and the user would be informed that the balance was not sufficient for this transaction.

```rust
#[derive(Debug)]
pub struct BankAccountError(String);
```

This error should implement `Display` and `Error` as well. 
Additionally, implementing the `From<&str>` trait will simplify the business logic that we'll be writing in the 
next sections.

```rust
impl Display for BankAccountError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.0)
    }
}

impl std::error::Error for BankAccountError {}

impl From<&str> for BankAccountError {
    fn from(message: &str) -> Self {
        BankAccountError(message.to_string())
    }
}
```

### External services

Business logic doesn't exist in a vacuum and external services may be needed for a variety of reasons.
We don't have much logic built yet, so this will initially just be a placeholder.
Let's add a couple of calls that will, for now, always return successfully.

```rust
pub struct BankAccountServices;

impl BankAccountServices {
    async fn atm_withdrawal(&self, atm_id: &str, amount: f64) -> Result<(), AtmError> {
        Ok(())
    }

    async fn validate_check(&self, account: &str, check: &str) -> Result<(), CheckingError> {
        Ok(())
    }
}
pub struct AtmError;
pub struct CheckingError;
```
