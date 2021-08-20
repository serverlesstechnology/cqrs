//!
//! A simple memory store for testing purposes only

pub use event_store::EventStore;
pub use query_store::QueryStore;

mod event_store;
mod query_store;
mod test;
