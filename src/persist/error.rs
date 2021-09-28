use crate::AggregateError;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum PersistenceError {
    OptimisticLockError,
    UnknownError(String),
}
impl Display for PersistenceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::OptimisticLockError => write!(f, "optimistic lock error"),
            PersistenceError::UnknownError(e) => write!(f, "{}", e.as_str()),
        }
    }
}
impl std::error::Error for PersistenceError {}

impl From<PersistenceError> for AggregateError {
    fn from(err: PersistenceError) -> Self {
        match err {
            PersistenceError::OptimisticLockError => {
                AggregateError::TechnicalError(err.to_string())
            }
            PersistenceError::UnknownError(e) => AggregateError::TechnicalError(e),
        }
    }
}
