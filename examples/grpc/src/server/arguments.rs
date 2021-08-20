use std::fmt::Debug;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "account-server",
    about = "BankAccount server"
)]
pub struct Arguments {
    /// Activate debug mode
    #[structopt(short, long)]
    pub debug: bool,

    /// Port number
    #[structopt(
        short = "p",
        long = "port",
        default_value = "50051"
    )]
    pub port: u32,
}
