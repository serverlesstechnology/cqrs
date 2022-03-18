//! Common persistence mechanisms.
//!
//! This module is used alongside one of the available repository crates:
//! - [postgres-es](https://crates.io/crates/postgres-es)
//! - [mysql-es](https://crates.io/crates/mysql-es)
//! - [dynamo-es](https://crates.io/crates/dynamo-es)
//!
//!
//!
pub use context::{EventStoreAggregateContext, ViewContext};
pub use error::PersistenceError;
pub use event_store::PersistedEventStore;
pub use queries::GenericQuery;
pub use repository::{PersistedEventRepository, ViewRepository};
pub use serialized_event::{SerializedEvent, SerializedSnapshot};
pub use upcaster::{
    EventUpcaster, SemanticVersion, SemanticVersionError, SemanticVersionEventUpcaster,
    SemanticVersionEventUpcasterFunc,
};

mod context;
mod error;
mod event_store;
mod queries;
mod repository;
mod serialized_event;
mod upcaster;

// Documentation items
#[doc(hidden)]
pub mod doc;
