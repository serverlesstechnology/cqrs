use cqrs_es2::TestFramework;

use crate::bank_account::{
    aggregate::BankAccount,
    commands::*,
    events::*,
};

type ThisTestFramework =
    TestFramework<BankAccountCommand, BankAccountEvent, BankAccount>;

#[test]
fn test_change_name() {
    ThisTestFramework::default()
        .given_no_previous_events()
        .when(BankAccountCommand::OpenBankAccount(
            OpenBankAccount {
                account_id: "John Doe".to_string(),
            },
        ))
        .then_expect_events(vec![
            BankAccountEvent::BankAccountOpened(BankAccountOpened {
                account_id: "John Doe".to_string(),
            }),
        ]);
}

#[test]
fn test_change_name_again() {
    ThisTestFramework::default()
        .given(vec![
            BankAccountEvent::BankAccountOpened(BankAccountOpened {
                account_id: "John Doe".to_string(),
            }),
        ])
        .when(BankAccountCommand::OpenBankAccount(
            OpenBankAccount {
                account_id: "John Doe".to_string(),
            },
        ))
        .then_expect_error("bank account is already open");
}
