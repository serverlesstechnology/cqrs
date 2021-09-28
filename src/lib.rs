#![forbid(unsafe_code)]
// #![deny(missing_docs)]
#![deny(clippy::all)]
#![warn(rust_2018_idioms)]
// #![warn(clippy::pedantic,missing_debug_implementations)]
//! # cqrs
//!
//! **A lightweight, opinionated CQRS and event sourcing framework targeting serverless architectures.**
//!
//! ![Build tag](https://codebuild.us-west-2.amazonaws.com/badges?uuid=eyJlbmNyeXB0ZWREYXRhIjoia3ZYcXozMjVZaFhoTldlUmhHemlWVm9LUjVaTC9LN3dSTFZpMkVTTmRycElkcGhJT3g2TUdtajZyRWZMd01xNktvUkNwLzdZYW15bzJkZldQMjJWZ1dNPSIsIml2UGFyYW1ldGVyU3BlYyI6InFORDNyaFFEQUNFQkE1NlUiLCJtYXRlcmlhbFNldFNlcmlhbCI6MX0%3D&branch=master)
//! [![Crates.io](https://img.shields.io/crates/v/cqrs-es)](https://crates.io/crates/cqrs-es)
//! [![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/cqrs-es)
//! ---
//!
//! ## Installation
//!
//! cqrs-es is available from Crates.io or Github.
//!
//! ```toml
//! [dependencies]
//! cqrs-es = "0.2.0"
//! ```
//!
//! ## Usage
//!
//! Documentation [is available here](https://doc.rust-cqrs.org) along with an introduction to CQRS and event sourcing.
//!
//! A demo application [is available here](https://github.com/serverlesstechnology/cqrs-demo).
//!
//! ## Todos
//!
//! - Event upcasters.
//! - Event serialization uses the event type as the root node of the JSON tree. This simplifies
//! deserialization but is non-standard.
//! - A persistence implementation for DynamoDb.
//! - A persistence implementation for MySql.

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
pub mod mem_store;

/// Test provides a test framework for building a resilient test base around aggregates.
/// A `TestFramework` should be used to build a comprehensive set of aggregate tests to verify
/// your application logic (aka business rules).
pub mod test;

// Query provides the basic downstream query objects needed to render queries (or "views") that
// describe the state of the system.
mod query;

/// Common persistence logic used for database-backed event stores.
pub mod persist;
