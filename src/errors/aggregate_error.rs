use std::{
    error,
    fmt::{
        Debug,
        Display,
        Formatter,
        Result as fmtResult,
    },
};

use serde::{
    Deserialize,
    Serialize,
};

use super::user_error_payload::UserErrorPayload;

/// The base error for the framework.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

impl error::Error for AggregateError {}

impl Display for AggregateError {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> fmtResult {
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
