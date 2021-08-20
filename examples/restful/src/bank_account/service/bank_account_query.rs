use log::debug;

use iron::{
    status,
    IronResult,
    Request,
    Response,
};
use router::Router;

use cqrs_es2::IQueryStore;

use super::super::stores::get_query_store;

use super::common::std_headers;

pub fn bank_account_query(req: &mut Request) -> IronResult<Response> {
    debug!("Received query '{:?}'", &req);

    let query_id = req
        .extensions
        .get::<Router>()
        .unwrap()
        .find("query_id")
        .unwrap_or("")
        .to_string();

    let mut query_store = get_query_store();

    let query = match query_store.load(&query_id) {
        Err(_e) => {
            return Ok(Response::with(status::BadRequest));
        },
        Ok(x) => x,
    };

    let body = serde_json::to_string(&query.payload).unwrap();

    let mut response = Response::with((status::Ok, body));
    response.headers = std_headers();

    Ok(response)
}
