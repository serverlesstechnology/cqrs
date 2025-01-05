use crate::PostgresEventRepository;
use cqrs_es::persist::PersistedEventStore;
use cqrs_es::CqrsFramework;

/// A convenience type for a CqrsFramework backed by
/// [PostgresStore](struct.PostgresStore.html).
pub type PostgresCqrs<A> = CqrsFramework<A, PersistedEventStore<PostgresEventRepository, A>>;
