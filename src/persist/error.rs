use crate::persist::SerializedEvent;
use crate::AggregateError;

/// Errors for implementations of a persistent event store.
#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    /// Optimistic locking conflict occurred while committing and aggregate.
    #[error("optimistic lock error")]
    OptimisticLockError,
    /// An error occurred connecting to the database.
    #[error("{0}")]
    ConnectionError(Box<dyn std::error::Error + Send + Sync + 'static>),
    /// Error occurred while attempting to deserialize data.
    #[error("{0}")]
    DeserializationError(Box<dyn std::error::Error + Send + Sync + 'static>),
    /// An unexpected error occurred while accessing the database.
    #[error("{0}")]
    UnknownError(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl<T: std::error::Error> From<PersistenceError> for AggregateError<T> {
    fn from(err: PersistenceError) -> Self {
        match err {
            PersistenceError::OptimisticLockError => Self::AggregateConflict,
            PersistenceError::ConnectionError(error) => Self::DatabaseConnectionError(error),
            PersistenceError::DeserializationError(error) => Self::DeserializationError(error),
            PersistenceError::UnknownError(error) => Self::UnexpectedError(error),
        }
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(err: serde_json::Error) -> Self {
        match err.classify() {
            serde_json::error::Category::Data | serde_json::error::Category::Syntax => {
                Self::DeserializationError(Box::new(err))
            }
            serde_json::error::Category::Io | serde_json::error::Category::Eof => {
                Self::UnknownError(Box::new(err))
            }
        }
    }
}

impl From<tokio::sync::mpsc::error::SendError<Result<SerializedEvent, Self>>> for PersistenceError {
    fn from(err: tokio::sync::mpsc::error::SendError<Result<SerializedEvent, Self>>) -> Self {
        Self::UnknownError(Box::new(err))
    }
}
