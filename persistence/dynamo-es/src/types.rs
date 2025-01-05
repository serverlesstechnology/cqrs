use cqrs_es::persist::PersistedEventStore;
use cqrs_es::CqrsFramework;

use crate::DynamoEventRepository;

/// A convenience type for a CqrsFramework backed by
/// [DynamoStore](struct.DynamoStore.html).
pub type DynamoCqrs<A> = CqrsFramework<A, PersistedEventStore<DynamoEventRepository, A>>;
