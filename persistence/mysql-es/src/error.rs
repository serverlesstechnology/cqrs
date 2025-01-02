use std::fmt::{Debug, Display, Formatter};

use cqrs_es::persist::PersistenceError;
use cqrs_es::AggregateError;
use sqlx::Error;

#[derive(Debug)]
pub enum MysqlAggregateError {
    OptimisticLock,
    ConnectionError(Box<dyn std::error::Error + Send + Sync + 'static>),
    DeserializationError(Box<dyn std::error::Error + Send + Sync + 'static>),
    UnknownError(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl Display for MysqlAggregateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MysqlAggregateError::OptimisticLock => write!(f, "optimistic lock error"),
            MysqlAggregateError::ConnectionError(error) => write!(f, "{}", error),
            MysqlAggregateError::DeserializationError(error) => write!(f, "{}", error),
            MysqlAggregateError::UnknownError(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for MysqlAggregateError {}

impl From<sqlx::Error> for MysqlAggregateError {
    fn from(err: sqlx::Error) -> Self {
        // TODO: improve error handling
        match &err {
            Error::Database(database_error) => {
                if let Some(code) = database_error.code() {
                    if code.as_ref() == "23000" {
                        return MysqlAggregateError::OptimisticLock;
                    }
                }
                MysqlAggregateError::UnknownError(Box::new(err))
            }
            Error::Io(_) | Error::Tls(_) => MysqlAggregateError::ConnectionError(Box::new(err)),
            _ => MysqlAggregateError::UnknownError(Box::new(err)),
        }
    }
}

impl<T: std::error::Error> From<MysqlAggregateError> for AggregateError<T> {
    fn from(err: MysqlAggregateError) -> Self {
        match err {
            MysqlAggregateError::OptimisticLock => AggregateError::AggregateConflict,
            MysqlAggregateError::DeserializationError(error) => {
                AggregateError::DeserializationError(error)
            }
            MysqlAggregateError::ConnectionError(error) => {
                AggregateError::DatabaseConnectionError(error)
            }
            MysqlAggregateError::UnknownError(error) => AggregateError::UnexpectedError(error),
        }
    }
}

impl From<serde_json::Error> for MysqlAggregateError {
    fn from(err: serde_json::Error) -> Self {
        match err.classify() {
            serde_json::error::Category::Data | serde_json::error::Category::Syntax => {
                MysqlAggregateError::DeserializationError(Box::new(err))
            }
            serde_json::error::Category::Io | serde_json::error::Category::Eof => {
                MysqlAggregateError::UnknownError(Box::new(err))
            }
        }
    }
}

impl From<MysqlAggregateError> for PersistenceError {
    fn from(err: MysqlAggregateError) -> Self {
        match err {
            MysqlAggregateError::OptimisticLock => PersistenceError::OptimisticLockError,
            MysqlAggregateError::ConnectionError(error) => PersistenceError::ConnectionError(error),
            MysqlAggregateError::DeserializationError(error) => {
                PersistenceError::DeserializationError(error)
            }
            MysqlAggregateError::UnknownError(error) => PersistenceError::UnknownError(error),
        }
    }
}
