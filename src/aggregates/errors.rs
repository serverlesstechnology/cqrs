use std::{
    collections::HashMap,
    error,
    fmt,
};

use serde::{
    Deserialize,
    Serialize,
};

/// The base error for the framework.
#[derive(Debug, PartialEq)]
pub enum AggregateError {
    /// The user has made an error, a String value contains a message
    /// to be delivered to the user.
    UserError(UserErrorPayload),
    /// A technical error was encountered that prevented the command
    /// from being applied to the aggregate. In general the
    /// accompanying message should be logged for investigation
    /// rather than returned to the user.
    TechnicalError(String),
}

/// Payload for an `AggregateError::UserError`, somewhat modeled on
/// the errors produced by the [`validator`](https://github.com/Keats/validator) package. This payload implements `Serialize`
/// with the intention of allowing the user to return this object as
/// the response payload.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UserErrorPayload {
    /// An optional code to indicate the a user-defined error.
    pub code: Option<String>,
    /// An optional message describing the error, meant to be
    /// returned to the user.
    pub message: Option<String>,
    /// Optional additional parameters for adding additional context
    /// to the error.
    pub params: Option<HashMap<String, String>>,
}

impl error::Error for AggregateError {}

impl fmt::Display for AggregateError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            AggregateError::TechnicalError(message) => {
                write!(f, "{}", message)
            },
            AggregateError::UserError(message) => {
                write!(f, "{}", message)
            },
        }
    }
}

impl fmt::Display for UserErrorPayload {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(
            f,
            "UserError - code: {:?}\n  message: {:?}\n params: {:?}",
            &self.code, &self.message, &self.params
        )
    }
}

impl AggregateError {
    /// Convenience function to construct a simple `UserError` from a
    /// `&str`.
    pub fn new(msg: &str) -> Self {
        AggregateError::UserError(UserErrorPayload {
            code: None,
            message: Some(msg.to_string()),
            params: None,
        })
    }
}

impl From<serde_json::error::Error> for AggregateError {
    fn from(err: serde_json::error::Error) -> Self {
        match err.classify() {
            serde_json::error::Category::Syntax => {
                AggregateError::new("invalid json")
            },
            serde_json::error::Category::Io |
            serde_json::error::Category::Data |
            serde_json::error::Category::Eof => {
                AggregateError::new("fail")
            },
        }
    }
}
