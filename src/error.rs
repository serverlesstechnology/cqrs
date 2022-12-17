use std::error;

/// The base error for the framework.
#[derive(Debug, thiserror::Error)]
pub enum AggregateError<T: error::Error> {
    /// This is the error returned when a user violates a business rule. The payload within
    /// `AggregateError::UserError` should be used to pass information to inform the user of
    /// the nature of problem.
    ///
    /// The `UserErrorPayload` struct has been provided as a reference implementation for this
    /// purpose.
    ///
    /// ### Handling
    /// In a Restful application this should translate to a 400 response status.
    #[error("{0}")]
    UserError(T),
    /// A command has been rejected due to a conflict with another command on the same aggregate
    /// instance. This is handled by optimistic locking in systems backed by an RDBMS.
    ///
    /// ### Handling
    /// In a Restful application this usually translates to a 503 or 429 response status, often with
    /// a [Retry-After response header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Retry-After)
    /// indicating that the user should try again.
    #[error("aggregate conflict")]
    AggregateConflict,
    /// A error occurred while attempting to read or write from a database.
    #[error("{0}")]
    DatabaseConnectionError(Box<dyn error::Error + Send + Sync + 'static>),
    /// A deserialization error occurred due to invalid JSON.
    #[error("{0}")]
    DeserializationError(Box<dyn error::Error + Send + Sync + 'static>),
    /// A technical error was encountered that prevented the command from being applied to the
    /// aggregate. In general the accompanying message should be logged for investigation rather
    /// than returned to the user.
    #[error("{0}")]
    UnexpectedError(Box<dyn error::Error + Send + Sync + 'static>),
}

impl<T: error::Error> From<serde_json::error::Error> for AggregateError<T> {
    fn from(err: serde_json::error::Error) -> Self {
        match err.classify() {
            serde_json::error::Category::Data | serde_json::error::Category::Syntax => {
                Self::DeserializationError(Box::new(err))
            }
            serde_json::error::Category::Io | serde_json::error::Category::Eof => {
                Self::UnexpectedError(Box::new(err))
            }
        }
    }
}
