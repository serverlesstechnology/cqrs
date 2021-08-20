use serde::{
    Deserialize,
    Serialize,
};
use std::fmt::Debug;

use cqrs_es2::ICommand;

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub enum BankAccountCommand {
    OpenBankAccount(OpenBankAccount),
    DepositMoney(DepositMoney),
    WithdrawMoney(WithdrawMoney),
    WriteCheck(WriteCheck),
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct OpenBankAccount {
    pub account_id: String,
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct DepositMoney {
    pub amount: f64,
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct WithdrawMoney {
    pub amount: f64,
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct WriteCheck {
    pub check_number: String,
    pub amount: f64,
}

impl ICommand for BankAccountCommand {}
