use serde::{
    Deserialize,
    Serialize,
};
use std::fmt::Debug;

use cqrs_es2::{
    EventContext,
    IEventConsumer,
    IQuery,
};

use super::super::{
    commands::BankAccountCommand,
    events::BankAccountEvent,
};

#[derive(
    Debug,
    PartialEq,
    Default,
    Clone,
    Serialize,
    Deserialize
)]
pub struct BankAccountQuery {
    pub account_id: Option<String>,
    pub balance: f64,
    pub written_checks: Vec<String>,
}

impl IQuery<BankAccountCommand, BankAccountEvent>
    for BankAccountQuery
{
    fn query_type() -> &'static str {
        "bank_account_query"
    }
}

impl IEventConsumer<BankAccountCommand, BankAccountEvent>
    for BankAccountQuery
{
    fn update(
        &mut self,
        event: &EventContext<BankAccountCommand, BankAccountEvent>,
    ) {
        match &event.payload {
            BankAccountEvent::BankAccountOpened(payload) => {
                self.account_id = Some(payload.account_id.clone());
            },
            BankAccountEvent::CustomerDepositedMoney(payload) => {
                self.balance = payload.balance;
            },
            BankAccountEvent::CustomerWithdrewCash(payload) => {
                self.balance = payload.balance;
            },
            BankAccountEvent::CustomerWroteCheck(payload) => {
                self.balance = payload.balance;
                self.written_checks
                    .push(payload.check_number.clone())
            },
        }
    }
}
