//! # example_impl
//!
//! A full example of a CQRS implementation for a `Customer`
//! aggregate. It serves the following purposes:
//!
//! - Document a usage scenario for the CQRS pattern
//! - Serve as shared resource for unit tests, doc tests, and
//!   integration tests

pub use aggregate::*;
pub use commands::*;
pub use events::*;
pub use queries::*;

mod aggregate;
mod commands;
mod events;
mod queries;

#[cfg(test)]
mod test;
