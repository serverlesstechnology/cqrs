use cqrs_es::Aggregate;
use serde::{Deserialize, Serialize};

use crate::domain::commands::BankAccountCommand;
use crate::domain::events::{BankAccountError, BankAccountEvent};
use crate::services::BankAccountServices;

#[derive(Serialize, Deserialize, Default)]
pub struct BankAccount {
    account_id: String,
    balance: f64,
}

impl Aggregate for BankAccount {
    // This identifier should be unique to the system.
    const TYPE: &'static str = "account";
    type Command = BankAccountCommand;
    type Event = BankAccountEvent;
    type Error = BankAccountError;
    type Services = BankAccountServices;

    // The aggregate logic goes here. Note that this will be the _bulk_ of a CQRS system
    // so expect to use helper functions elsewhere to keep the code clean.
    async fn handle(
        &self,
        command: Self::Command,
        services: &Self::Services,
    ) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            BankAccountCommand::OpenAccount { account_id } => {
                Ok(vec![BankAccountEvent::AccountOpened { account_id }])
            }
            BankAccountCommand::DepositMoney { amount } => {
                let balance = self.balance + amount;
                Ok(vec![BankAccountEvent::CustomerDepositedMoney {
                    amount,
                    balance,
                }])
            }
            BankAccountCommand::WithdrawMoney { amount, atm_id } => {
                let balance = self.balance - amount;
                if balance < 0_f64 {
                    return Err("funds not available".into());
                }
                if services
                    .services
                    .atm_withdrawal(&atm_id, amount)
                    .await
                    .is_err()
                {
                    return Err("atm rule violation".into());
                }
                Ok(vec![BankAccountEvent::CustomerWithdrewCash {
                    amount,
                    balance,
                }])
            }
            BankAccountCommand::WriteCheck {
                check_number,
                amount,
            } => {
                let balance = self.balance - amount;
                if balance < 0_f64 {
                    return Err("funds not available".into());
                }
                if services
                    .services
                    .validate_check(&self.account_id, &check_number)
                    .await
                    .is_err()
                {
                    return Err("check invalid".into());
                }
                Ok(vec![BankAccountEvent::CustomerWroteCheck {
                    check_number,
                    amount,
                    balance,
                }])
            }
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            BankAccountEvent::AccountOpened { account_id } => {
                self.account_id = account_id;
            }
            BankAccountEvent::CustomerDepositedMoney { amount: _, balance }
            | BankAccountEvent::CustomerWithdrewCash { amount: _, balance }
            | BankAccountEvent::CustomerWroteCheck {
                check_number: _,
                amount: _,
                balance,
            } => {
                self.balance = balance;
            }
        }
    }
}

// The aggregate tests are the most important part of a CQRS system.
// The simplicity and flexibility of these tests are a good part of what
// makes an event sourced system so friendly to changing business requirements.
#[cfg(test)]
mod aggregate_tests {
    use async_trait::async_trait;
    use std::sync::Mutex;

    use cqrs_es::test::TestFramework;

    use crate::domain::aggregate::BankAccount;
    use crate::domain::commands::BankAccountCommand;
    use crate::domain::events::BankAccountEvent;
    use crate::services::{AtmError, BankAccountApi, BankAccountServices, CheckingError};

    // A test framework that will apply our events and command
    // and verify that the logic works as expected.
    type AccountTestFramework = TestFramework<BankAccount>;

    #[test]
    fn test_deposit_money() {
        let expected = BankAccountEvent::CustomerDepositedMoney {
            amount: 200.0,
            balance: 200.0,
        };
        let command = BankAccountCommand::DepositMoney { amount: 200.0 };
        let services = BankAccountServices::new(Box::new(MockBankAccountServices::default()));
        // Obtain a new test framework
        AccountTestFramework::with(services)
            // In a test case with no previous events
            .given_no_previous_events()
            // Wnen we fire this command
            .when(command)
            // then we expect these results
            .then_expect_events(vec![expected]);
    }

    #[test]
    fn test_deposit_money_with_balance() {
        let previous = BankAccountEvent::CustomerDepositedMoney {
            amount: 200.0,
            balance: 200.0,
        };
        let expected = BankAccountEvent::CustomerDepositedMoney {
            amount: 200.0,
            balance: 400.0,
        };
        let command = BankAccountCommand::DepositMoney { amount: 200.0 };
        let services = BankAccountServices::new(Box::new(MockBankAccountServices::default()));

        AccountTestFramework::with(services)
            // Given this previously applied event
            .given(vec![previous])
            // When we fire this command
            .when(command)
            // Then we expect this resultant event
            .then_expect_events(vec![expected]);
    }

    #[test]
    fn test_withdraw_money() {
        let previous = BankAccountEvent::CustomerDepositedMoney {
            amount: 200.0,
            balance: 200.0,
        };
        let expected = BankAccountEvent::CustomerWithdrewCash {
            amount: 100.0,
            balance: 100.0,
        };
        let services = MockBankAccountServices::default();
        services.set_atm_withdrawal_response(Ok(()));
        let command = BankAccountCommand::WithdrawMoney {
            amount: 100.0,
            atm_id: "ATM34f1ba3c".to_string(),
        };

        AccountTestFramework::with(BankAccountServices::new(Box::new(services)))
            .given(vec![previous])
            .when(command)
            .then_expect_events(vec![expected]);
    }

    #[test]
    fn test_withdraw_money_client_error() {
        let previous = BankAccountEvent::CustomerDepositedMoney {
            amount: 200.0,
            balance: 200.0,
        };
        let services = MockBankAccountServices::default();
        services.set_atm_withdrawal_response(Err(AtmError));
        let command = BankAccountCommand::WithdrawMoney {
            amount: 100.0,
            atm_id: "ATM34f1ba3c".to_string(),
        };

        let services = BankAccountServices::new(Box::new(services));
        AccountTestFramework::with(services)
            .given(vec![previous])
            .when(command)
            .then_expect_error_message("atm rule violation");
    }

    #[test]
    fn test_withdraw_money_funds_not_available() {
        let command = BankAccountCommand::WithdrawMoney {
            amount: 200.0,
            atm_id: "ATM34f1ba3c".to_string(),
        };

        let services = BankAccountServices::new(Box::new(MockBankAccountServices::default()));
        AccountTestFramework::with(services)
            .given_no_previous_events()
            .when(command)
            // Here we expect an error rather than any events
            .then_expect_error_message("funds not available");
    }

    #[test]
    fn test_wrote_check() {
        let previous = BankAccountEvent::CustomerDepositedMoney {
            amount: 200.0,
            balance: 200.0,
        };
        let expected = BankAccountEvent::CustomerWroteCheck {
            check_number: "1170".to_string(),
            amount: 100.0,
            balance: 100.0,
        };
        let services = MockBankAccountServices::default();
        services.set_validate_check_response(Ok(()));
        let services = BankAccountServices::new(Box::new(services));
        let command = BankAccountCommand::WriteCheck {
            check_number: "1170".to_string(),
            amount: 100.0,
        };

        AccountTestFramework::with(services)
            .given(vec![previous])
            .when(command)
            .then_expect_events(vec![expected]);
    }

    #[test]
    fn test_wrote_check_bad_check() {
        let previous = BankAccountEvent::CustomerDepositedMoney {
            amount: 200.0,
            balance: 200.0,
        };
        let services = MockBankAccountServices::default();
        services.set_validate_check_response(Err(CheckingError));
        let services = BankAccountServices::new(Box::new(services));
        let command = BankAccountCommand::WriteCheck {
            check_number: "1170".to_string(),
            amount: 100.0,
        };

        AccountTestFramework::with(services)
            .given(vec![previous])
            .when(command)
            .then_expect_error_message("check invalid");
    }

    #[test]
    fn test_wrote_check_funds_not_available() {
        let command = BankAccountCommand::WriteCheck {
            check_number: "1170".to_string(),
            amount: 100.0,
        };

        let services = BankAccountServices::new(Box::new(MockBankAccountServices::default()));
        AccountTestFramework::with(services)
            .given_no_previous_events()
            .when(command)
            .then_expect_error_message("funds not available");
    }

    pub struct MockBankAccountServices {
        atm_withdrawal_response: Mutex<Option<Result<(), AtmError>>>,
        validate_check_response: Mutex<Option<Result<(), CheckingError>>>,
    }

    impl Default for MockBankAccountServices {
        fn default() -> Self {
            Self {
                atm_withdrawal_response: Mutex::new(None),
                validate_check_response: Mutex::new(None),
            }
        }
    }

    impl MockBankAccountServices {
        fn set_atm_withdrawal_response(&self, response: Result<(), AtmError>) {
            *self.atm_withdrawal_response.lock().unwrap() = Some(response);
        }
        fn set_validate_check_response(&self, response: Result<(), CheckingError>) {
            *self.validate_check_response.lock().unwrap() = Some(response);
        }
    }

    #[async_trait]
    impl BankAccountApi for MockBankAccountServices {
        async fn atm_withdrawal(&self, _atm_id: &str, _amount: f64) -> Result<(), AtmError> {
            self.atm_withdrawal_response.lock().unwrap().take().unwrap()
        }

        async fn validate_check(
            &self,
            _account_id: &str,
            _check_number: &str,
        ) -> Result<(), CheckingError> {
            self.validate_check_response.lock().unwrap().take().unwrap()
        }
    }
}
