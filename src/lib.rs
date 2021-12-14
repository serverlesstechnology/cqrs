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

// Aggregate module holds the central traits that define the fundamental component of CQRS.
mod aggregate;

// Event module provides the abstract domain events and associated wrapper.
mod event;

// Store holds the abstact `EventStore` trait as well as an in-memory and Postgres implementation.
mod store;

// Cqrs provides the base framework and associated logic for processing loading aggregates via an
// event store and subsequently processing commands.
mod cqrs;

// Aggregate error
mod error;

// Query provides the basic downstream query objects needed to render queries (or "views") that
// describe the state of the system.
mod query;

// Documentation items
#[doc(hidden)]
pub mod doc;

/// An in-memory event store suitable for local testing.
///
/// A backing store is necessary for any application to store and retrieve the generated events.
/// This in-memory store is useful for application development and integration tests that do not
/// require persistence after running.
///
/// ```
/// # use cqrs_es::doc::MyAggregate;
/// use cqrs_es::CqrsFramework;
/// use cqrs_es::mem_store::MemStore;
///
/// let store = MemStore::<MyAggregate>::default();
/// let cqrs = CqrsFramework::new(store, vec![]);
/// ```
pub mod mem_store;

/// Test provides a test framework for building a resilient test base around aggregates.
/// A `TestFramework` should be used to build a comprehensive set of aggregate tests to verify
/// your application logic.
///
/// ```
/// # use cqrs_es::test::TestFramework;
/// # use cqrs_es::doc::{Customer, CustomerEvent, CustomerCommand};
/// type CustomerTestFramework = TestFramework<Customer>;
///
/// CustomerTestFramework::default()
///     .given_no_previous_events()
///     .when(CustomerCommand::AddCustomerName{
///             changed_name: "John Doe".to_string()
///         })
///     .then_expect_events(vec![
///         CustomerEvent::NameAdded{
///             changed_name: "John Doe".to_string()
///         }]);
///
/// CustomerTestFramework::default()
///     .given(vec![
///         CustomerEvent::NameAdded {
///             changed_name: "John Doe".to_string()
///         }])
///     .when(CustomerCommand::AddCustomerName {
///             changed_name: "John Doe".to_string()
///         })
///     .then_expect_error("a name has already been added for this customer")
/// ```
pub mod test;
