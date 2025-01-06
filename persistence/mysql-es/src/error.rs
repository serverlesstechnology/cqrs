use cqrs_es::persist::PersistenceError;
use cqrs_es::AggregateError;
use sqlx::Error;

#[derive(Debug, thiserror::Error)]
pub enum MysqlAggregateError {
    #[error("optimistic lock error")]
    OptimisticLock,
    #[error(transparent)]
    ConnectionError(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error(transparent)]
    DeserializationError(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error(transparent)]
    UnknownError(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl From<sqlx::Error> for MysqlAggregateError {
    fn from(err: sqlx::Error) -> Self {
        // TODO: improve error handling
        match &err {
            Error::Database(database_error) => {
                if let Some(code) = database_error.code() {
                    if code.as_ref() == "23000" {
                        return Self::OptimisticLock;
                    }
                }
                Self::UnknownError(Box::new(err))
            }
            Error::Io(_) | Error::Tls(_) => Self::ConnectionError(Box::new(err)),
            _ => Self::UnknownError(Box::new(err)),
        }
    }
}

impl<T: std::error::Error> From<MysqlAggregateError> for AggregateError<T> {
    fn from(err: MysqlAggregateError) -> Self {
        match err {
            MysqlAggregateError::OptimisticLock => Self::AggregateConflict,
            MysqlAggregateError::DeserializationError(error) => Self::DeserializationError(error),
            MysqlAggregateError::ConnectionError(error) => Self::DatabaseConnectionError(error),
            MysqlAggregateError::UnknownError(error) => Self::UnexpectedError(error),
        }
    }
}

impl From<serde_json::Error> for MysqlAggregateError {
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

impl From<MysqlAggregateError> for PersistenceError {
    fn from(err: MysqlAggregateError) -> Self {
        match err {
            MysqlAggregateError::OptimisticLock => Self::OptimisticLockError,
            MysqlAggregateError::ConnectionError(error) => Self::ConnectionError(error),
            MysqlAggregateError::DeserializationError(error) => Self::DeserializationError(error),
            MysqlAggregateError::UnknownError(error) => Self::UnknownError(error),
        }
    }
}
