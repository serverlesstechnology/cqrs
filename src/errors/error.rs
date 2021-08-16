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

use super::user_error::UserError;

/// The base error for the framework.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Error {
    /// The user has made an error, a String value contains a message
    /// to be delivered to the user.
    UserError(UserError),

    /// A technical error was encountered that prevented the command
    /// from being applied to the aggregate. In general the
    /// accompanying message should be logged for investigation
    /// rather than returned to the user.
    TechnicalError(String),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> fmtResult {
        match self {
            Error::TechnicalError(message) => {
                write!(f, "{}", message)
            },
            Error::UserError(message) => {
                write!(f, "{}", message)
            },
        }
    }
}

impl Error {
    /// Convenience function to construct a simple `UserError` from a
    /// `&str`.
    pub fn new(msg: &str) -> Self {
        Error::UserError(UserError {
            code: None,
            message: Some(msg.to_string()),
            params: None,
        })
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Self {
        match err.classify() {
            serde_json::error::Category::Syntax => {
                Error::new("invalid json")
            },
            serde_json::error::Category::Io |
            serde_json::error::Category::Data |
            serde_json::error::Category::Eof => Error::new("fail"),
        }
    }
}
