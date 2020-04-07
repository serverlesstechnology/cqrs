use std::collections::hash_map::RandomState;
use std::collections::HashMap;

use chrono::Utc;

/// A [`MetadataSupplier`] will add metadata to events as they are produced by the aggregate.
pub trait MetadataSupplier
{
    /// Provides the Metadata for a new event.
    /// Currently this takes no input which is quite limiting of its' usefulness.
    /// This will likely change in the future.
    fn supply(&self) -> HashMap<String, String>;
}

/// A simple [`MetadataSupplier`] that adds no information.
pub struct NoopMetadataSupplier {}

impl Default for NoopMetadataSupplier {
    fn default() -> Self {
        NoopMetadataSupplier {}
    }
}

impl MetadataSupplier for NoopMetadataSupplier {
    fn supply(&self) -> HashMap<String, String, RandomState> { Default::default() }
}

/// A simple [`MetadataSupplier`] that adds the current time. Note that these times are not
/// guaranteed to be unique.
pub struct TimeMetadataSupplier {}

impl Default for TimeMetadataSupplier {
    fn default() -> Self {
        TimeMetadataSupplier {}
    }
}

impl MetadataSupplier for TimeMetadataSupplier {
    fn supply(&self) -> HashMap<String, String, RandomState> {
        let mut metadata = HashMap::new();
        metadata.insert("time".to_string(), Utc::now().to_rfc3339());
        metadata
    }
}

