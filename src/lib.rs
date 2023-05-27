#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![warn(rust_2018_idioms)]
// #![warn(clippy::pedantic,missing_debug_implementations)]
#![doc = include_str!("../README.md")]
//!
pub use crate::aggregate::*;
pub use crate::cqrs::*;
pub use crate::error::*;
pub use crate::event::*;
pub use crate::query::*;
pub use crate::store::*;

mod aggregate;
mod cqrs;
mod error;
mod event;
mod query;
mod store;

#[doc(hidden)]
pub mod doc;

/// An in-memory event store suitable for local testing.
///
/// A backing store is necessary for any application to store and retrieve the generated events.
/// This in-memory store is useful for application development and integration tests that do not
/// require persistence after running.
///
/// ```
/// # use cqrs_es::doc::{MyAggregate, MyService};
/// use cqrs_es::CqrsFramework;
/// use cqrs_es::mem_store::MemStore;
///
/// let store = MemStore::<MyAggregate>::default();
/// let service = MyService::default();
/// let cqrs = CqrsFramework::new(store, vec![], service);
/// ```
pub mod mem_store;

pub mod persist;

pub mod test;
