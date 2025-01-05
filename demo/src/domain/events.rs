use cqrs_es::DomainEvent;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

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

impl DomainEvent for BankAccountEvent {
    fn event_type(&self) -> String {
        match self {
            BankAccountEvent::AccountOpened { .. } => "AccountOpened".to_string(),
            BankAccountEvent::CustomerDepositedMoney { .. } => "CustomerDepositedMoney".to_string(),
            BankAccountEvent::CustomerWithdrewCash { .. } => "CustomerWithdrewCash".to_string(),
            BankAccountEvent::CustomerWroteCheck { .. } => "CustomerWroteCheck".to_string(),
        }
    }

    fn event_version(&self) -> String {
        "1.0".to_string()
    }
}

#[derive(Debug)]
pub struct BankAccountError(String);

impl From<&str> for BankAccountError {
    fn from(msg: &str) -> Self {
        Self(msg.to_string())
    }
}

impl Display for BankAccountError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for BankAccountError {}
