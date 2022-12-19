use async_trait::async_trait;
use axum::extract::{FromRequest, RequestParts};
use std::collections::HashMap;
use std::convert::Infallible;

// This is a custom Axum extension that builds metadata from the inbound request.
pub struct MetadataExtension(pub HashMap<String, String>);

const USER_AGENT_HDR: &str = "User-Agent";

#[async_trait]
impl<B: Send> FromRequest<B> for MetadataExtension {
    type Rejection = Infallible;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // Here we are including the current date/time, the uri that was called and the user-agent
        // in a HashMap that we will submit as metadata with the command.
        let mut metadata = HashMap::default();
        metadata.insert("time".to_string(), chrono::Utc::now().to_rfc3339());
        metadata.insert("uri".to_string(), req.uri().to_string());
        if let Some(user_agent) = req.headers().get(USER_AGENT_HDR) {
            if let Ok(value) = user_agent.to_str() {
                metadata.insert(USER_AGENT_HDR.to_string(), value.to_string());
            }
        }
        Ok(MetadataExtension(metadata))
    }
}
