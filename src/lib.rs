// #![deny(missing_docs)]

#[cfg(test)]
extern crate static_assertions;


/// Aggregate module holds the central traits that define the fundamental component of CQRS.
pub mod aggregate;

/// Event module provides the abstract domain events and associated wrapper.
pub mod event;

/// Store holds the abstact `EventStore` trait as well as an in-memory and Postgres implementation.
pub mod store;

/// Command module holds the `Command` trait which defines the only object that can make any
/// modifications to the state of an aggregate.
pub mod command;

/// Cqrs provides the base framework and associated logic for processing loading aggregates via an
/// event store and subsequently processing commands.
pub mod cqrs;

/// Config has additional suppliers of metadata to be included with the committed events.
pub mod config;

/// Test provides a test framework for building a resilient test base around aggregates.
pub mod test;

/// Tools provides a simple postgres view repository to simplify loading and updating views.
pub mod tools;

/// View provides the basic downstream query objects needed to render queries (or "views") that
/// describe the state of the system.
pub mod view;
