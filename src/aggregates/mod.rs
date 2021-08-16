//! # aggregates
//!
//! A central location for `Aggregate` interfaces

pub use aggregate_context::AggregateContext;
pub use i_aggregate::IAggregate;

mod aggregate_context;
mod i_aggregate;
