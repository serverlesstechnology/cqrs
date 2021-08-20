use serde::{
    Deserialize,
    Serialize,
};
use std::fmt::Debug;

use cqrs_es2::{
    Error,
    IAggregate,
    ICommandHandler,
    IEventHandler,
};

use super::super::{
    commands::BankAccountCommand,
    events::{
        BankAccountEvent,
        BankAccountOpened,
        CustomerDepositedMoney,
        CustomerWithdrewCash,
        CustomerWroteCheck,
    },
};

#[derive(
    Debug,
    PartialEq,
    Default,
    Clone,
    Serialize,
    Deserialize
)]
pub struct BankAccount {
    account_id: String,
    balance: f64,
}

impl IAggregate<BankAccountCommand, BankAccountEvent>
    for BankAccount
{
    fn aggregate_type() -> &'static str {
        "bank_account"
    }
}

impl ICommandHandler<BankAccountCommand, BankAccountEvent>
    for BankAccount
{
    fn handle(
        &self,
        command: BankAccountCommand,
    ) -> Result<Vec<BankAccountEvent>, Error> {
        match command {
            BankAccountCommand::OpenBankAccount(payload) => {
                if !self.account_id.is_empty() {
                    return Err(Error::new(
                        "bank account is already open",
                    ));
                }

                let event_payload = BankAccountOpened {
                    account_id: payload.account_id,
                };

                Ok(vec![
                    BankAccountEvent::BankAccountOpened(
                        event_payload,
                    ),
                ])
            },
            BankAccountCommand::DepositMoney(payload) => {
                let balance = self.balance + payload.amount;

                let event_payload = CustomerDepositedMoney {
                    amount: payload.amount,
                    balance,
                };

                Ok(vec![
                    BankAccountEvent::CustomerDepositedMoney(
                        event_payload,
                    ),
                ])
            },
            BankAccountCommand::WithdrawMoney(payload) => {
                let balance = self.balance - payload.amount;

                if balance < 0_f64 {
                    return Err(Error::new("funds not available"));
                }

                let event_payload = CustomerWithdrewCash {
                    amount: payload.amount,
                    balance,
                };

                Ok(vec![
                    BankAccountEvent::CustomerWithdrewCash(
                        event_payload,
                    ),
                ])
            },
            BankAccountCommand::WriteCheck(payload) => {
                let balance = self.balance - payload.amount;

                if balance < 0_f64 {
                    return Err(Error::new("funds not available"));
                }

                let event_payload = CustomerWroteCheck {
                    check_number: payload.check_number,
                    amount: payload.amount,
                    balance,
                };

                Ok(vec![
                    BankAccountEvent::CustomerWroteCheck(
                        event_payload,
                    ),
                ])
            },
        }
    }
}

impl IEventHandler<BankAccountEvent> for BankAccount {
    fn apply(
        &mut self,
        event: &BankAccountEvent,
    ) {
        match event {
            BankAccountEvent::BankAccountOpened(e) => {
                self.account_id = e.account_id.clone();
            },
            BankAccountEvent::CustomerDepositedMoney(e) => {
                self.balance = e.balance;
            },
            BankAccountEvent::CustomerWithdrewCash(e) => {
                self.balance = e.balance;
            },
            BankAccountEvent::CustomerWroteCheck(e) => {
                self.balance = e.balance;
            },
        }
    }
}
