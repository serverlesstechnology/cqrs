#![forbid(unsafe_code)]
#![deny(clippy::all)]

use iron::Iron;
use router::Router;

use log::info;

mod bank_account;
mod cqrs;

use bank_account::{
    bank_account_command,
    bank_account_query,
};

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

fn main() {
    setup_logger().unwrap();

    let mut router = Router::new();
    router.get(
        "/account/:query_id",
        bank_account_query,
        "account_query",
    );

    router.post(
        "/account/:command_type/:aggregate_id",
        bank_account_command,
        "account_command",
    );

    info!("Starting server at http://localhost:3030");

    Iron::new(router)
        .http("localhost:3030")
        .unwrap();
}
