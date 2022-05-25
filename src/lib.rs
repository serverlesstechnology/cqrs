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
/// This module provides a test framework for building a resilient test base around aggregates.
/// A `TestFramework` should be used to build a comprehensive set of aggregate tests to verify
/// your application logic.
///
/// ```rust
/// # use cqrs_es::test::TestFramework;
/// # use cqrs_es::doc::{Customer, CustomerEvent, CustomerCommand, CustomerService};
/// # fn test() {
/// type CustomerTestFramework = TestFramework<Customer>;
///
/// CustomerTestFramework::with(CustomerService::default())
///     .given_no_previous_events()
///     .when(CustomerCommand::AddCustomerName{
///             name: "John Doe".to_string()
///         })
///     .then_expect_events(vec![
///         CustomerEvent::NameAdded{
///             name: "John Doe".to_string()
///         }]);
/// # }
/// ```
pub mod test;
