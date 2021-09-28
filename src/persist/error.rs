use std::fmt::{Display, Formatter};

use crate::AggregateError;

/// Errors for implementations of a persistent event store.
#[derive(Debug)]
pub enum PersistenceError {
    /// Optimistic locking conflict occurred while committing and aggregate.
    OptimisticLockError,
    /// An error occurred connecting to the database.
    ConnectionError(String),
    /// An unexpected error occurred while accessing the database.
    UnknownError(String),
}

impl Display for PersistenceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::OptimisticLockError => write!(f, "optimistic lock error"),
            PersistenceError::ConnectionError(msg) => write!(f, "{}", msg),
            PersistenceError::UnknownError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<PersistenceError> for AggregateError {
    fn from(err: PersistenceError) -> Self {
        match err {
            PersistenceError::ConnectionError(msg) => AggregateError::TechnicalError(msg),
            PersistenceError::UnknownError(msg) => AggregateError::TechnicalError(msg),
            PersistenceError::OptimisticLockError => AggregateError::AggregateConflict,
        }
    }
}
