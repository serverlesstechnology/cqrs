use std::fmt;
use std::collections::HashMap;
use std::error;
use serde::{Serialize,Deserialize};

/// The base error for the framework.
#[derive(Debug, PartialEq)]
pub enum AggregateError {
    /// This is the error returned when a user violates a business rule. The information within
    /// the `UserErrorPayload` should be used to inform the user of their error.
    ///
    /// ### Handling
    /// In a Restful application this should translate to a 400 response status.
    UserError(UserErrorPayload),
    /// A command has been rejected due to a conflict with another command on the same aggregate
    /// instance. This is handled by optimistic locking in systems backed by an RDBMS.
    ///
    /// ### Handling
    /// In a Restful application this usually translates to a 500 response status.
    ///
    /// If the call comes from a server this should be retried immediately.
    AggregateConflict,
    /// A technical error was encountered that prevented the command from being applied to the
    /// aggregate. In general the accompanying message should be logged for investigation rather
    /// than returned to the user.
    ///
    /// ### Handling
    /// In a Restful application this usually translates to a 500 or 503 response status.
    ///
    /// In a production system this may indicate a serious error and should be investigated.
    TechnicalError(String),
}

/// Payload for an `AggregateError::UserError`, somewhat modeled on the errors produced by the
/// [`validator`](https://github.com/Keats/validator) package. This payload implements `Serialize`
/// with the intention of allowing the user to return this object as the response payload.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UserErrorPayload {
    /// An optional code to indicate the a user-defined error.
    pub code: Option<String>,
    /// An optional message describing the error, meant to be returned to the user.
    pub message: Option<String>,
    /// Optional additional parameters for adding additional context to the error.
    pub params: Option<HashMap<String, String>>,
}

impl error::Error for AggregateError {}

impl fmt::Display for AggregateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregateError::TechnicalError(message) => write!(f, "{}", message),
            AggregateError::UserError(message) => write!(f, "{}", message),
            AggregateError::AggregateConflict => write!(f, "aggregate conflict"),
        }
    }
}

impl fmt::Display for UserErrorPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UserError - code: {:?}\n  message: {:?}\n params: {:?}",
            &self.code, &self.message, &self.params
        )
    }
}

impl AggregateError {
    /// A convenience function to construct a simple `UserError` with a user message.
    ///
    /// ```
    /// # use cqrs_es::AggregateError;
    /// let error = AggregateError::new("user already exists");
    /// ```
    pub fn new(msg: &str) -> Self {
        AggregateError::UserError(UserErrorPayload {
            code: None,
            message: Some(msg.to_string()),
            params: None,
        })
    }
    /// A convenience function to construct a simple `UserError` with a user message and error code.
    ///
    /// ```
    /// # use cqrs_es::AggregateError;
    /// let error = AggregateError::new_with_code("user already exists", "USER_EXISTS");
    /// ```
    pub fn new_with_code(msg: &str, code: &str) -> Self {
        AggregateError::UserError(UserErrorPayload {
            code: Some(code.to_string()),
            message: Some(msg.to_string()),
            params: None,
        })
    }
}

impl From<serde_json::error::Error> for AggregateError {
    fn from(err: serde_json::error::Error) -> Self {
        match err.classify() {
            serde_json::error::Category::Syntax => AggregateError::new("invalid json"),
            serde_json::error::Category::Io
            | serde_json::error::Category::Data
            | serde_json::error::Category::Eof => AggregateError::new("fail"),
        }
    }
}
