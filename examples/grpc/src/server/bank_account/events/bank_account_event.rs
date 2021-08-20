use serde::{
    Deserialize,
    Serialize,
};
use std::fmt::Debug;

use cqrs_es2::IEvent;

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub enum BankAccountEvent {
    BankAccountOpened(BankAccountOpened),
    CustomerDepositedMoney(CustomerDepositedMoney),
    CustomerWithdrewCash(CustomerWithdrewCash),
    CustomerWroteCheck(CustomerWroteCheck),
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct BankAccountOpened {
    pub account_id: String,
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct CustomerDepositedMoney {
    pub amount: f64,
    pub balance: f64,
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct CustomerWithdrewCash {
    pub amount: f64,
    pub balance: f64,
}

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize
)]
pub struct CustomerWroteCheck {
    pub check_number: String,
    pub amount: f64,
    pub balance: f64,
}

impl IEvent for BankAccountEvent {}
