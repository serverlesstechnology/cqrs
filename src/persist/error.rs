use std::fmt::{Display, Formatter};

use crate::AggregateError;

#[derive(Debug)]
pub enum PersistenceError {
    OptimisticLockError,
    ConnectionError(String),
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
