//! Common persistence mechanisms.
//!
//! This module is used alongside one of the available repository crates:
//! - [postgres-es](https://crates.io/crates/postgres-es)
//! - [mysql-es](https://crates.io/crates/mysql-es)
//! - [dynamo-es](https://crates.io/crates/dynamo-es)
//!
//!
//!
pub use context::EventStoreAggregateContext;
pub use error::PersistenceError;
pub use event_repository::PersistedEventRepository;
pub use event_store::PersistedEventStore;
pub use event_stream::{ReplayStream,ReplayFeed};
pub use generic_query::{GenericQuery, QueryErrorHandler};
pub use replay::{QueryReplay};
pub use serialized_event::{SerializedEvent, SerializedSnapshot};
pub use upcaster::{
    EventUpcaster, SemanticVersion, SemanticVersionError, SemanticVersionEventUpcaster,
    SemanticVersionEventUpcasterFunc,
};
pub use view_repository::{ViewContext, ViewRepository};

mod context;
mod error;
mod event_repository;
mod event_store;
mod event_stream;
mod generic_query;
mod replay;
mod serialized_event;
mod upcaster;
mod view_repository;

// Documentation items
#[doc(hidden)]
pub mod doc;
