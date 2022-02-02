use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error;
use std::fmt;

/// The base error for the framework.
#[derive(Debug)]
pub enum AggregateError<T: std::error::Error> {
    /// This is the error returned when a user violates a business rule. The payload within
    /// `AggregateError::UserError` should be used to pass information to inform the user of
    /// the nature of problem.
    ///
    /// The `UserErrorPayload` struct has been provided as a reference implementation for this
    /// purpose.
    ///
    /// ### Handling
    /// In a Restful application this should translate to a 400 response status.
    UserError(T),
    /// A command has been rejected due to a conflict with another command on the same aggregate
    /// instance. This is handled by optimistic locking in systems backed by an RDBMS.
    ///
    /// ### Handling
    /// In a Restful application this usually translates to a 500 response status.
    ///
    /// If the call comes from a server this should be retried immediately.
    AggregateConflict,
    /// A error occurred while attempting to read or write from a database.
    ///
    DatabaseConnectionError(Box<dyn std::error::Error + Send + Sync + 'static>),
    /// A deserialization error occurred due to invalid JSON.
    ///
    DeserializationError(Box<dyn std::error::Error + Send + Sync + 'static>),
    /// A technical error was encountered that prevented the command from being applied to the
    /// aggregate. In general the accompanying message should be logged for investigation rather
    /// than returned to the user.
    ///
    /// ### Handling
    /// In a Restful application this usually translates to a 500 or 503 response status.
    ///
    /// In a production system this may indicate a serious error and should be investigated.
    TechnicalError(Box<dyn std::error::Error + Send + Sync + 'static>),
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

impl<T: std::error::Error> error::Error for AggregateError<T> {}

impl<T: std::error::Error> fmt::Display for AggregateError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregateError::UserError(message) => write!(f, "{}", message),
            AggregateError::AggregateConflict => write!(f, "aggregate conflict"),
            AggregateError::DeserializationError(error) => write!(f, "{}", error),
            AggregateError::DatabaseConnectionError(error) => write!(f, "{}", error),
            AggregateError::TechnicalError(error) => write!(f, "{}", error),
        }
    }
}

impl<T: std::error::Error> AggregateError<T> {
    /// A convenience function to construct a simple `AggregateError::UserError` with the given payload.
    ///
    /// ```
    /// # use cqrs_es::{AggregateError, UserErrorPayload};
    /// let error = AggregateError::new(UserErrorPayload {
    ///             code: None,
    ///             message: Some("user already exists".to_string()),
    ///             params: None,
    ///         });
    /// ```
    pub fn new(error_payload: T) -> Self {
        AggregateError::UserError(error_payload)
    }
}

impl error::Error for UserErrorPayload {}

impl fmt::Display for UserErrorPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match &self.message {
            None => "unknown error",
            Some(message) => message.as_ref(),
        };
        write!(f, "{}", message)
    }
}

impl From<&str> for AggregateError<UserErrorPayload> {
    fn from(message: &str) -> Self {
        AggregateError::UserError(UserErrorPayload {
            code: None,
            message: Some(message.to_string()),
            params: None,
        })
    }
}

impl<T: std::error::Error> From<serde_json::error::Error> for AggregateError<T> {
    fn from(err: serde_json::error::Error) -> Self {
        match err.classify() {
            serde_json::error::Category::Data | serde_json::error::Category::Syntax => {
                AggregateError::DeserializationError(Box::new(err))
            }
            serde_json::error::Category::Io | serde_json::error::Category::Eof => {
                AggregateError::TechnicalError(Box::new(err))
            }
        }
    }
}
