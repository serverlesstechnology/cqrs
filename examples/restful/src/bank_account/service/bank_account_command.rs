use log::debug;
use std::{
    collections::HashMap,
    io::Read,
};

use iron::{
    status,
    IronResult,
    Request,
    Response,
};
use router::Router;

use cqrs_es2::Error;

use super::super::stores::get_event_store;

use super::common::std_headers;

pub fn bank_account_command(
    req: &mut Request
) -> IronResult<Response> {
    debug!("Received command '{:?}'", &req);

    let params = req.extensions.get::<Router>().unwrap();

    let command_type = params
        .find("command_type")
        .unwrap_or("");

    let aggregate_id = params
        .find("aggregate_id")
        .unwrap_or("");

    let mut payload = String::new();

    req.body
        .read_to_string(&mut payload)
        .unwrap();

    let result = match command_type {
        "openBankAccount" => {
            process_command("OpenBankAccount", aggregate_id, payload)
        },
        "depositMoney" => {
            process_command("DepositMoney", aggregate_id, payload)
        },
        "withdrawMoney" => {
            process_command("WithdrawMoney", aggregate_id, payload)
        },
        "writeCheck" => {
            process_command("WriteCheck", aggregate_id, payload)
        },
        _ => return Ok(Response::with(status::NotFound)),
    };
    match result {
        Ok(_) => Ok(Response::with(status::NoContent)),
        Err(err) => {
            let err_payload = match &err {
                Error::UserError(e) => {
                    serde_json::to_string(e).unwrap()
                },
                Error::TechnicalError(e) => e.clone(),
            };
            let mut response =
                Response::with((status::BadRequest, err_payload));
            response.headers = std_headers();
            Ok(response)
        },
    }
}

fn process_command(
    payload_type: &str,
    aggregate_id: &str,
    payload: String,
) -> Result<(), Error> {
    let event_ser = format!("{{\"{}\":{}}}", payload_type, payload);

    let payload = match serde_json::from_str(event_ser.as_str()) {
        Ok(payload) => payload,
        Err(err) => {
            return Err(Error::TechnicalError(err.to_string()));
        },
    };

    let mut event_store = get_event_store();

    let mut metadata = HashMap::new();
    metadata.insert(
        "time".to_string(),
        chrono::Utc::now().to_rfc3339(),
    );

    event_store.execute_with_metadata(aggregate_id, payload, metadata)
}
