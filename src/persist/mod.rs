//! Common persistence mechanisms to be used with one of the repository crates:
//! - postgres-es
//! - mysql-es
//! - dynamodb-es
//!
//!
//!
pub use context::{EventStoreAggregateContext, QueryContext, SnapshotStoreAggregateContext};
pub use error::PersistenceError;
pub use event_store::PersistedEventStore;
pub use queries::GenericQuery;
pub use repository::{PersistedEventRepository, ViewRepository};
pub use serialized_event::{SerializedEvent, SerializedSnapshot};
pub use snapshot_store::PersistedSnapshotStore;
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
mod snapshot_store;
mod upcaster;

// Documentation items
#[doc(hidden)]
pub mod doc;
