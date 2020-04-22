#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::all)]
// #![warn(clippy::pedantic)]
//! # srvrls - a lightweight serverless framework
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
//! cqrs-es = "0.0.9"
//! ```
//! 
//! Or for a specific branch
//! ```toml
//! [dependencies]
//! cqrs-es = { git = "https://github.com/serverlesstechnology/cqrs.git", branch = "master"}
//! ```
//! 
//! ## Opinions
//! 
//! - Aggregate persistence is via event sourcing only.
//! - Metadata is implemented only as a `HashMap<String,String>`. 
//! Further, the `MetadataSupplier` that the user provides has no insight into the event or aggregate that 
//! it supplies metadata for. This may be changed.
//! - JSON serialization only.
//! - Generics are preferred over boxed traits.
//! - Persistence is implemented through a Postgres database.
//! 
//! ## Todos/research
//! 
//! - Event upcasters.
//! - Some additional framework around `GenericViewRepository` to simplify event replay.
//! - Explore options for increasing the usefulness of `MetadataSupplier`.
//! - Event serialization uses the event type as the root node of the JSON tree. This simplifies
//! deserialization but is non-standard.
//! - Paging for PostgresEventStore
//! - Persistence implementation for DynamoDb.
//!

#[cfg(test)]
extern crate static_assertions;
// Aggregate module holds the central traits that define the fundamental component of CQRS.
mod aggregate;

// Event module provides the abstract domain events and associated wrapper.
mod event;

// Store holds the abstact `EventStore` trait as well as an in-memory and Postgres implementation.
mod store;

// Command module holds the `Command` trait which defines the only object that can make any
// modifications to the state of an aggregate.
mod command;

// Cqrs provides the base framework and associated logic for processing loading aggregates via an
// event store and subsequently processing commands.
mod cqrs;

// Config has additional suppliers of metadata to be included with the committed events.
mod config;

/// An in-memory event store suitable for local testing.
pub mod mem_store;

/// Test provides a test framework for building a resilient test base around aggregates.
pub mod test;


/// View provides the basic downstream query objects needed to render queries (or "views") that
/// describe the state of the system.
pub mod view;

pub use crate::aggregate::*;
pub use crate::command::*;
pub use crate::cqrs::*;
pub use crate::config::*;
pub use crate::event::*;
pub use crate::store::*;
