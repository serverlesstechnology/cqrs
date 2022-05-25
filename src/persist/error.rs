use crate::persist::SerializedEvent;
use crate::AggregateError;
use std::fmt::{Display, Formatter};

/// Errors for implementations of a persistent event store.
#[derive(Debug)]
pub enum PersistenceError {
    /// Optimistic locking conflict occurred while committing and aggregate.
    OptimisticLockError,
    /// An error occurred connecting to the database.
    ConnectionError(Box<dyn std::error::Error + Send + Sync + 'static>),
    /// Error occurred while attempting to deserialize data.
    DeserializationError(Box<dyn std::error::Error + Send + Sync + 'static>),
    /// An unexpected error occurred while accessing the database.
    UnknownError(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl Display for PersistenceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::OptimisticLockError => write!(f, "optimistic lock error"),
            PersistenceError::ConnectionError(error) => write!(f, "{}", error),
            PersistenceError::DeserializationError(error) => write!(f, "{}", error),
            PersistenceError::UnknownError(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for PersistenceError {}

impl<T: std::error::Error> From<PersistenceError> for AggregateError<T> {
    fn from(err: PersistenceError) -> Self {
        match err {
            PersistenceError::OptimisticLockError => AggregateError::AggregateConflict,
            PersistenceError::ConnectionError(error) => {
                AggregateError::DatabaseConnectionError(error)
            }
            PersistenceError::DeserializationError(error) => {
                AggregateError::DeserializationError(error)
            }
            PersistenceError::UnknownError(error) => AggregateError::UnexpectedError(error),
        }
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(err: serde_json::Error) -> Self {
        match err.classify() {
            serde_json::error::Category::Data | serde_json::error::Category::Syntax => {
                PersistenceError::DeserializationError(Box::new(err))
            }
            serde_json::error::Category::Io | serde_json::error::Category::Eof => {
                PersistenceError::UnknownError(Box::new(err))
            }
        }
    }
}

impl From<tokio::sync::mpsc::error::SendError<Result<SerializedEvent, PersistenceError>>>
    for PersistenceError
{
    fn from(
        err: tokio::sync::mpsc::error::SendError<Result<SerializedEvent, PersistenceError>>,
    ) -> Self {
        PersistenceError::UnknownError(Box::new(err))
    }
}
