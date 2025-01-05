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
            DynamoAggregateError::OptimisticLock => write!(f, "optimistic lock error"),
            DynamoAggregateError::MissingAttribute(attribute) => write!(f, "missing attribute: {}", attribute),
            DynamoAggregateError::ConnectionError(msg) => write!(f, "{}", msg),
            DynamoAggregateError::DeserializationError(msg) => write!(f, "{}", msg),
            DynamoAggregateError::UnknownError(msg) => write!(f, "{}", msg),
            DynamoAggregateError::TransactionListTooLong(length) => write!(f, "Too many operations: {}, DynamoDb supports only up to 25 operations per transactions", length),
        }
    }
}

impl std::error::Error for DynamoAggregateError {}

impl<T: std::error::Error> From<DynamoAggregateError> for AggregateError<T> {
    fn from(error: DynamoAggregateError) -> Self {
        match error {
            DynamoAggregateError::OptimisticLock => AggregateError::AggregateConflict,
            DynamoAggregateError::ConnectionError(err) => {
                AggregateError::DatabaseConnectionError(err)
            }
            DynamoAggregateError::DeserializationError(err) => {
                AggregateError::DeserializationError(err)
            }
            DynamoAggregateError::TransactionListTooLong(_) => {
                AggregateError::UnexpectedError(Box::new(error))
            }
            DynamoAggregateError::MissingAttribute(err) => AggregateError::UnexpectedError(
                Box::new(DynamoAggregateError::MissingAttribute(err)),
            ),
            DynamoAggregateError::UnknownError(err) => AggregateError::UnexpectedError(err),
        }
    }
}

impl From<serde_json::Error> for DynamoAggregateError {
    fn from(err: serde_json::Error) -> Self {
        DynamoAggregateError::UnknownError(Box::new(err))
    }
}

impl From<SdkError<TransactWriteItemsError>> for DynamoAggregateError {
    fn from(error: SdkError<TransactWriteItemsError>) -> Self {
        if let SdkError::ServiceError(err) = &error {
            if let TransactWriteItemsError::TransactionCanceledException(cancellation) = err.err() {
                for reason in cancellation.cancellation_reasons() {
                    if reason.code() == Some("ConditionalCheckFailed") {
                        return DynamoAggregateError::OptimisticLock;
                    }
                }
            }
        }
        DynamoAggregateError::UnknownError(Box::new(error))
    }
}

impl From<SdkError<QueryError>> for DynamoAggregateError {
    fn from(error: SdkError<QueryError>) -> Self {
        unknown_error(error)
    }
}

impl From<BuildError> for DynamoAggregateError {
    fn from(error: BuildError) -> Self {
        DynamoAggregateError::UnknownError(Box::new(error))
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
            DynamoAggregateError::OptimisticLock => PersistenceError::OptimisticLockError,
            DynamoAggregateError::ConnectionError(err) => PersistenceError::ConnectionError(err),
            DynamoAggregateError::DeserializationError(err) => {
                PersistenceError::DeserializationError(err)
            }
            DynamoAggregateError::TransactionListTooLong(_) => {
                PersistenceError::UnknownError(Box::new(error))
            }
            DynamoAggregateError::MissingAttribute(err) => PersistenceError::UnknownError(
                Box::new(DynamoAggregateError::MissingAttribute(err)),
            ),
            DynamoAggregateError::UnknownError(err) => PersistenceError::UnknownError(err),
        }
    }
}
