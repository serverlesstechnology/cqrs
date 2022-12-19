use std::fmt::{Debug, Display, Formatter};

use cqrs_es::persist::PersistenceError;
use cqrs_es::AggregateError;
use sqlx::Error;

#[derive(Debug)]
pub enum PostgresAggregateError {
    OptimisticLock,
    ConnectionError(Box<dyn std::error::Error + Send + Sync + 'static>),
    DeserializationError(Box<dyn std::error::Error + Send + Sync + 'static>),
    UnknownError(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl Display for PostgresAggregateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PostgresAggregateError::OptimisticLock => write!(f, "optimistic lock error"),
            PostgresAggregateError::UnknownError(error) => write!(f, "{}", error),
            PostgresAggregateError::DeserializationError(error) => write!(f, "{}", error),
            PostgresAggregateError::ConnectionError(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for PostgresAggregateError {}

impl From<sqlx::Error> for PostgresAggregateError {
    fn from(err: sqlx::Error) -> Self {
        // TODO: improve error handling
        match &err {
            Error::Database(database_error) => {
                if let Some(code) = database_error.code() {
                    if code.as_ref() == "23505" {
                        return PostgresAggregateError::OptimisticLock;
                    }
                }
                PostgresAggregateError::UnknownError(Box::new(err))
            }
            Error::Io(_) | Error::Tls(_) => PostgresAggregateError::ConnectionError(Box::new(err)),
            _ => PostgresAggregateError::UnknownError(Box::new(err)),
        }
    }
}

impl<T: std::error::Error> From<PostgresAggregateError> for AggregateError<T> {
    fn from(err: PostgresAggregateError) -> Self {
        match err {
            PostgresAggregateError::OptimisticLock => AggregateError::AggregateConflict,
            PostgresAggregateError::ConnectionError(error) => {
                AggregateError::DatabaseConnectionError(error)
            }
            PostgresAggregateError::DeserializationError(error) => {
                AggregateError::DeserializationError(error)
            }
            PostgresAggregateError::UnknownError(error) => AggregateError::UnexpectedError(error),
        }
    }
}

impl From<serde_json::Error> for PostgresAggregateError {
    fn from(err: serde_json::Error) -> Self {
        match err.classify() {
            serde_json::error::Category::Data | serde_json::error::Category::Syntax => {
                PostgresAggregateError::DeserializationError(Box::new(err))
            }
            serde_json::error::Category::Io | serde_json::error::Category::Eof => {
                PostgresAggregateError::UnknownError(Box::new(err))
            }
        }
    }
}

impl From<PostgresAggregateError> for PersistenceError {
    fn from(err: PostgresAggregateError) -> Self {
        match err {
            PostgresAggregateError::OptimisticLock => PersistenceError::OptimisticLockError,
            PostgresAggregateError::ConnectionError(error) => {
                PersistenceError::ConnectionError(error)
            }
            PostgresAggregateError::DeserializationError(error) => {
                PersistenceError::UnknownError(error)
            }
            PostgresAggregateError::UnknownError(error) => PersistenceError::UnknownError(error),
        }
    }
}
