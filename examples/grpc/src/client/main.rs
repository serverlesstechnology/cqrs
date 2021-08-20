#[path = "../proto/bank_account_api.rs"]
mod bank_account_api;

use bank_account_api::{
    bank_account_client::BankAccountClient,
    OpenBankAccountRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel =
        tonic::transport::Channel::from_static("http://[::1]:50051")
            .connect()
            .await?;

    let mut client = BankAccountClient::new(channel);

    let request = tonic::Request::new(OpenBankAccountRequest {
        account_id: "new-account-1".to_string(),
    });

    // sending request and waiting for response
    let response = client
        .open_account(request)
        .await?
        .into_inner();
    println!("RESPONSE={:?}", response);
    Ok(())
}
