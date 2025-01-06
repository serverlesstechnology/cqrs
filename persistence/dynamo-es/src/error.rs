use std::fmt::{Debug, Display, Formatter};

use aws_sdk_dynamodb::error::{BuildError, SdkError};
use aws_sdk_dynamodb::operation::query::QueryError;
use aws_sdk_dynamodb::operation::scan::ScanError;
use aws_sdk_dynamodb::operation::transact_write_items::TransactWriteItemsError;
use cqrs_es::persist::PersistenceError;
use cqrs_es::AggregateError;
use serde::de::StdError;

#[derive(Debug)]
pub enum DynamoAggregateError {
    OptimisticLock,
    ConnectionError(Box<dyn std::error::Error + Send + Sync + 'static>),
    DeserializationError(Box<dyn std::error::Error + Send + Sync + 'static>),
    TransactionListTooLong(usize),
    MissingAttribute(String),
    UnknownError(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl Display for DynamoAggregateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OptimisticLock => write!(f, "optimistic lock error"),
            Self::MissingAttribute(attribute) => write!(f, "missing attribute: {attribute}"),
            Self::ConnectionError(msg) => write!(f, "{msg}"),
            Self::DeserializationError(msg) => write!(f, "{msg}"),
            Self::UnknownError(msg) => write!(f, "{msg}"),
            Self::TransactionListTooLong(length) => write!(f, "Too many operations: {length}, DynamoDb supports only up to 25 operations per transactions"),
        }
    }
}

impl std::error::Error for DynamoAggregateError {}

impl<T: std::error::Error> From<DynamoAggregateError> for AggregateError<T> {
    fn from(error: DynamoAggregateError) -> Self {
        match error {
            DynamoAggregateError::OptimisticLock => Self::AggregateConflict,
            DynamoAggregateError::ConnectionError(err) => Self::DatabaseConnectionError(err),
            DynamoAggregateError::DeserializationError(err) => Self::DeserializationError(err),
            DynamoAggregateError::TransactionListTooLong(_) => {
                Self::UnexpectedError(Box::new(error))
            }
            DynamoAggregateError::MissingAttribute(err) => {
                Self::UnexpectedError(Box::new(DynamoAggregateError::MissingAttribute(err)))
            }
            DynamoAggregateError::UnknownError(err) => Self::UnexpectedError(err),
        }
    }
}

impl From<serde_json::Error> for DynamoAggregateError {
    fn from(err: serde_json::Error) -> Self {
        Self::UnknownError(Box::new(err))
    }
}

impl From<SdkError<TransactWriteItemsError>> for DynamoAggregateError {
    fn from(error: SdkError<TransactWriteItemsError>) -> Self {
        if let SdkError::ServiceError(err) = &error {
            if let TransactWriteItemsError::TransactionCanceledException(cancellation) = err.err() {
                for reason in cancellation.cancellation_reasons() {
                    if reason.code() == Some("ConditionalCheckFailed") {
                        return Self::OptimisticLock;
                    }
                }
            }
        }
        Self::UnknownError(Box::new(error))
    }
}

impl From<SdkError<QueryError>> for DynamoAggregateError {
    fn from(error: SdkError<QueryError>) -> Self {
        unknown_error(error)
    }
}

impl From<BuildError> for DynamoAggregateError {
    fn from(error: BuildError) -> Self {
        Self::UnknownError(Box::new(error))
    }
}

impl From<SdkError<ScanError>> for DynamoAggregateError {
    fn from(error: SdkError<ScanError>) -> Self {
        unknown_error(error)
    }
}

fn unknown_error<T: StdError + Send + Sync + 'static>(error: SdkError<T>) -> DynamoAggregateError {
    DynamoAggregateError::UnknownError(Box::new(error))
}

impl From<DynamoAggregateError> for PersistenceError {
    fn from(error: DynamoAggregateError) -> Self {
        match error {
            DynamoAggregateError::OptimisticLock => Self::OptimisticLockError,
            DynamoAggregateError::ConnectionError(err) => Self::ConnectionError(err),
            DynamoAggregateError::DeserializationError(err) => Self::DeserializationError(err),
            DynamoAggregateError::TransactionListTooLong(_) => Self::UnknownError(Box::new(error)),
            DynamoAggregateError::MissingAttribute(err) => {
                Self::UnknownError(Box::new(DynamoAggregateError::MissingAttribute(err)))
            }
            DynamoAggregateError::UnknownError(err) => Self::UnknownError(err),
        }
    }
}
