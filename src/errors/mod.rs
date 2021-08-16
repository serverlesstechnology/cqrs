//! # errors
//!
//! A central location for errors and error handling

pub use aggregate_error::AggregateError;
pub use user_error_payload::UserErrorPayload;

mod aggregate_error;
mod user_error_payload;
