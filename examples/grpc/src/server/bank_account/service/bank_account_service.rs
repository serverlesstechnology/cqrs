use log::debug;

use tonic::{
    Request,
    Response,
    Status,
};

use cqrs_es2::IQueryStore;

use crate::bank_account_api::{
    bank_account_server::BankAccount,
    BankAccountQueryRequest,
    BankAccountSummaryResponse,
    CommandResponse,
    DepositMoneyRequest,
    OpenBankAccountRequest,
    WithdrawMoneyRequest,
    WriteCheckRequest,
};

use super::super::{
    commands::*,
    stores::{
        get_event_store,
        get_query_store,
    },
};

#[derive(Default)]
pub struct BankAccountService {}

#[tonic::async_trait]
impl BankAccount for BankAccountService {
    async fn open_account(
        &self,
        request: Request<OpenBankAccountRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        let req = request.get_ref();

        debug!("{:?}", req);

        match get_event_store()
            .await
            .unwrap()
            .execute(
                req.account_id.as_str(),
                BankAccountCommand::OpenBankAccount(
                    OpenBankAccount {
                        account_id: req.account_id.clone(),
                    },
                ),
            )
            .await
        {
            Ok(_) => {
                Ok(Response::new(CommandResponse {
                    is_successful: true,
                }))
            },
            Err(err) => Err(Status::aborted(err.to_string())),
        }
    }

    async fn deposit_money(
        &self,
        request: Request<DepositMoneyRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        let req = request.get_ref();

        debug!("{:?}", req);

        match get_event_store()
            .await
            .unwrap()
            .execute(
                req.account_id.as_str(),
                BankAccountCommand::DepositMoney(DepositMoney {
                    amount: req.amount,
                }),
            )
            .await
        {
            Ok(_) => {
                Ok(Response::new(CommandResponse {
                    is_successful: true,
                }))
            },
            Err(err) => Err(Status::aborted(err.to_string())),
        }
    }

    async fn withdraw_money(
        &self,
        request: Request<WithdrawMoneyRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        let req = request.get_ref();

        debug!("{:?}", req);

        match get_event_store()
            .await
            .unwrap()
            .execute(
                req.account_id.as_str(),
                BankAccountCommand::WithdrawMoney(WithdrawMoney {
                    amount: req.amount,
                }),
            )
            .await
        {
            Ok(_) => {
                Ok(Response::new(CommandResponse {
                    is_successful: true,
                }))
            },
            Err(err) => Err(Status::aborted(err.to_string())),
        }
    }

    async fn write_check(
        &self,
        request: Request<WriteCheckRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        let req = request.get_ref();

        debug!("{:?}", req);

        match get_event_store()
            .await
            .unwrap()
            .execute(
                req.account_id.as_str(),
                BankAccountCommand::WriteCheck(WriteCheck {
                    check_number: req.check_number.to_string(),
                    amount: req.amount,
                }),
            )
            .await
        {
            Ok(_) => {
                Ok(Response::new(CommandResponse {
                    is_successful: true,
                }))
            },
            Err(err) => Err(Status::aborted(err.to_string())),
        }
    }

    async fn get_account_summary(
        &self,
        request: Request<BankAccountQueryRequest>,
    ) -> Result<Response<BankAccountSummaryResponse>, Status> {
        let req = request.get_ref();

        debug!("{:?}", req);

        let context = match get_query_store()
            .await
            .unwrap()
            .load(req.account_id.as_str())
            .await
        {
            Ok(x) => x,
            Err(e) => return Err(Status::aborted(e.to_string())),
        };

        let payload = context.payload;

        Ok(Response::new(
            BankAccountSummaryResponse {
                account_id: match payload.account_id {
                    None => "".to_string(),
                    Some(x) => x,
                },
                balance: payload.balance,
                written_checks: payload
                    .written_checks
                    .iter()
                    .map(|x| x.parse::<i64>().unwrap())
                    .collect(),
            },
        ))
    }
}
