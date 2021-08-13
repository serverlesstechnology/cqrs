#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![warn(rust_2018_idioms)]
// #![warn(clippy::pedantic,missing_debug_implementations)]
//! # cqrs-es2
//!
//! **A lightweight, opinionated CQRS and event sourcing framework
//! targeting serverless architectures.**
//!
//! [![Publish](https://github.com/brgirgis/cqrs-es2/actions/workflows/crates-io.yml/badge.svg)](https://github.com/brgirgis/cqrs-es2/actions/workflows/crates-io.yml)
//! [![Test](https://github.com/brgirgis/cqrs-es2/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/brgirgis/cqrs-es2/actions/workflows/rust-ci.yml)
//! [![Crates.io](https://img.shields.io/crates/v/cqrs-es2)](https://crates.io/crates/cqrs-es2)
//! [![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/cqrs-es2)
//! ---
//!
//! ## Installation
//!
//! cqrs-es2 is available from Crates.io or Github.
//!
//! ```toml
//! [dependencies]
//! cqrs-es2 = "0.2.4"
//! ```
//!
//! ## Usage
//!
//! Documentation [is available here](https://doc.rust-cqrs.org) along with an introduction to CQRS and event sourcing.
//!
//! A demo application [is available here](https://github.com/brgirgis/cqrs-es2-demo).
//!
//! ## Todos
//!
//! - Event upcasters.
//! - Event serialization uses the event type as the root node of the
//!   JSON tree. This simplifies
//! deserialization but is non-standard.
//! - A persistence implementation for DynamoDb.
//! - A persistence implementation for MySql.

pub use crate::{
    aggregates::*,
    events::*,
    framework::*,
    queries::*,
    stores::*,
};

// Aggregates module holds the central traits that define the
// fundamental component of CQRS.
mod aggregates;

// Events module provides the abstract domain events and associated
// wrapper.
mod events;

// Stores module holds the abstract `EventStore` trait as well as an
// in-memory implementation.
mod stores;

// Queries module provides the basic downstream query objects needed
// to render queries (or "views") that describe the state of the
// system.
mod queries;

// Framework provides the base framework and associated logic for
// processing loading aggregates via an event store and subsequently
// processing commands.
mod framework;

// Documentation items
#[doc(hidden)]
pub mod doc;

/// Test provides a test framework for building a resilient test base
/// around aggregates. A `TestFramework` should be used to build a
/// comprehensive set of aggregate tests to verify your application
/// logic (aka business rules).
pub mod test;
