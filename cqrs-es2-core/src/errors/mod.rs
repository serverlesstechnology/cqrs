//! # errors
//!
//! A central location for errors and error handling

pub use error::Error;
pub use user_error::UserError;

mod error;
mod user_error;
