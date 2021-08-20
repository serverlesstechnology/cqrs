fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/bank_account_api.proto")?;
    Ok(())
}
