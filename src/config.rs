use std::collections::hash_map::RandomState;
use std::collections::HashMap;

use chrono::Utc;

pub trait MetadataSupplier
{
    fn supply(&self) -> HashMap<String, String>;
}

pub struct NoopMetadataSupplier {}

impl Default for NoopMetadataSupplier {
    fn default() -> Self {
        NoopMetadataSupplier {}
    }
}

impl MetadataSupplier for NoopMetadataSupplier {
    fn supply(&self) -> HashMap<String, String, RandomState> { Default::default() }
}


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

