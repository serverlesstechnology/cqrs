#![forbid(unsafe_code)]
use log::info;

// arguments
use arguments::Arguments;
use structopt::StructOpt;

// service
use bank_account::BankAccountService;

// server
use bank_account_api::bank_account_server::BankAccountServer;
use tonic::transport::Server;

mod arguments;
mod bank_account;
#[path = "../proto/bank_account_api.rs"]
mod bank_account_api;
mod cqrs;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger().unwrap();

    let args = Arguments::from_args();

    let addr = format!("[::1]:{}", args.port)
        .parse()
        .unwrap();

    let account_service = BankAccountService::default();
    info!("Server listening on {}", addr);

    Server::builder()
        .add_service(BankAccountServer::new(account_service))
        .serve(addr)
        .await?;

    Ok(())
}
