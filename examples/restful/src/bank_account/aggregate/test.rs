use cqrs_es2::TestFramework;

use super::super::{
    commands::{
        BankAccountCommand,
        DepositMoney,
        WithdrawMoney,
        WriteCheck,
    },
    events::{
        BankAccountEvent,
        CustomerDepositedMoney,
        CustomerWithdrewCash,
        CustomerWroteCheck,
    },
};

use super::bank_account::BankAccount;

type BankAccountTestFramework =
    TestFramework<BankAccountCommand, BankAccountEvent, BankAccount>;

#[test]
fn test_deposit_money() {
    let expected = BankAccountEvent::CustomerDepositedMoney(
        CustomerDepositedMoney {
            amount: 200.0,
            balance: 200.0,
        },
    );

    BankAccountTestFramework::default()
        .given_no_previous_events()
        .when(BankAccountCommand::DepositMoney(
            DepositMoney { amount: 200.0 },
        ))
        .then_expect_events(vec![expected]);
}

#[test]
fn test_deposit_money_with_balance() {
    let previous = BankAccountEvent::CustomerDepositedMoney(
        CustomerDepositedMoney {
            amount: 200.0,
            balance: 200.0,
        },
    );

    let expected = BankAccountEvent::CustomerDepositedMoney(
        CustomerDepositedMoney {
            amount: 200.0,
            balance: 400.0,
        },
    );

    BankAccountTestFramework::default()
        .given(vec![previous])
        .when(BankAccountCommand::DepositMoney(
            DepositMoney { amount: 200.0 },
        ))
        .then_expect_events(vec![expected]);
}

#[test]
fn test_withdraw_money() {
    let previous = BankAccountEvent::CustomerDepositedMoney(
        CustomerDepositedMoney {
            amount: 200.0,
            balance: 200.0,
        },
    );

    let expected = BankAccountEvent::CustomerWithdrewCash(
        CustomerWithdrewCash {
            amount: 100.0,
            balance: 100.0,
        },
    );

    BankAccountTestFramework::default()
        .given(vec![previous])
        .when(BankAccountCommand::WithdrawMoney(
            WithdrawMoney { amount: 100.0 },
        ))
        .then_expect_events(vec![expected]);
}

#[test]
fn test_withdraw_money_funds_not_available() {
    BankAccountTestFramework::default()
        .given_no_previous_events()
        .when(BankAccountCommand::WithdrawMoney(
            WithdrawMoney { amount: 200.0 },
        ))
        .then_expect_error("funds not available")
}

#[test]
fn test_wrote_check() {
    let previous = BankAccountEvent::CustomerDepositedMoney(
        CustomerDepositedMoney {
            amount: 200.0,
            balance: 200.0,
        },
    );

    let expected =
        BankAccountEvent::CustomerWroteCheck(CustomerWroteCheck {
            check_number: "1170".to_string(),
            amount: 100.0,
            balance: 100.0,
        });

    BankAccountTestFramework::default()
        .given(vec![previous])
        .when(BankAccountCommand::WriteCheck(
            WriteCheck {
                check_number: "1170".to_string(),
                amount: 100.0,
            },
        ))
        .then_expect_events(vec![expected]);
}

#[test]
fn test_wrote_check_funds_not_available() {
    BankAccountTestFramework::default()
        .given_no_previous_events()
        .when(BankAccountCommand::WriteCheck(
            WriteCheck {
                check_number: "1170".to_string(),
                amount: 100.0,
            },
        ))
        .then_expect_error("funds not available")
}
