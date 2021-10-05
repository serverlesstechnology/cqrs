#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![warn(rust_2018_idioms)]
// #![warn(clippy::pedantic,missing_debug_implementations)]
//! # cqrs
//!
//! **A lightweight, opinionated CQRS and event sourcing framework targeting serverless architectures.**
//!
//! Command Query Responsibility Segregation (CQRS) is a pattern in
//! [Domain Driven Design](https://martinfowler.com/tags/domain%20driven%20design.html)
//! that uses separate write and read models for application objects and interconnects them with events.
//! Event sourcing uses the generated events as the source of truth for the
//! state of the application.
//!
//! Together these provide a number of benefits:
//! - Removes coupling between tests and application logic allowing limitless refactoring.
//! - Greater isolation of the [aggregate](https://martinfowler.com/bliki/DDD_Aggregate.html).
//! - Ability to create views that more accurately model our business environment.
//! - A horizontally scalable read path.
//!
//!
//! Things that could be quite helpful:
//! - [User guide](https://doc.rust-cqrs.org) along with an introduction to CQRS and event sourcing.
//! - [Demo application](https://github.com/serverlesstechnology/cqrs-demo) using the warp http server.
//!
//!
pub use crate::aggregate::*;
pub use crate::cqrs::*;
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

// Documentation items
#[doc(hidden)]
pub mod doc;

/// An in-memory event store suitable for local testing.
///
/// A backing store is necessary for any application to store and retrieve the generated events.
/// This in-memory store is useful for application development and integration tests that do not
/// require persistence after running.
pub mod mem_store;

/// Test provides a test framework for building a resilient test base around aggregates.
/// A `TestFramework` should be used to build a comprehensive set of aggregate tests to verify
/// your application logic (aka business rules).
pub mod test;

// Query provides the basic downstream query objects needed to render queries (or "views") that
// describe the state of the system.
mod query;


