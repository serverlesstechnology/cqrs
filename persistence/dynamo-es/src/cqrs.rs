use cqrs_es::persist::PersistedEventStore;
use cqrs_es::{Aggregate, CqrsFramework, Query};

use crate::{DynamoCqrs, DynamoEventRepository};

/// A convenience function for creating a CqrsFramework from a DynamoDb client
/// and queries.
pub fn dynamodb_cqrs<A>(
    dynamo_client: aws_sdk_dynamodb::client::Client,
    query_processor: Vec<Box<dyn Query<A>>>,
    services: A::Services,
) -> DynamoCqrs<A>
where
    A: Aggregate,
{
    let repo = DynamoEventRepository::new(dynamo_client);
    let store = PersistedEventStore::new_event_store(repo);
    CqrsFramework::new(store, query_processor, services)
}

/// A convenience function for creating a CqrsFramework using an aggregate store.
pub fn dynamodb_aggregate_cqrs<A>(
    dynamo_client: aws_sdk_dynamodb::client::Client,
    query_processor: Vec<Box<dyn Query<A>>>,
    services: A::Services,
) -> DynamoCqrs<A>
where
    A: Aggregate,
{
    let repo = DynamoEventRepository::new(dynamo_client);
    let store = PersistedEventStore::new_aggregate_store(repo);
    CqrsFramework::new(store, query_processor, services)
}

/// A convenience function for creating a CqrsFramework using a snapshot store.
pub fn dynamodb_snapshot_cqrs<A>(
    dynamo_client: aws_sdk_dynamodb::client::Client,
    query_processor: Vec<Box<dyn Query<A>>>,
    snapshot_size: usize,
    services: A::Services,
) -> DynamoCqrs<A>
where
    A: Aggregate,
{
    let repo = DynamoEventRepository::new(dynamo_client);
    let store = PersistedEventStore::new_snapshot_store(repo, snapshot_size);
    CqrsFramework::new(store, query_processor, services)
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::cqrs::dynamodb_cqrs;
    use crate::testing::tests::{test_dynamodb_client, TestQueryRepository, TestServices};
    use crate::DynamoViewRepository;

    #[tokio::test]
    async fn test_valid_cqrs_framework() {
        let client = test_dynamodb_client().await;
        let view_repo = DynamoViewRepository::new("test_query", client.clone());
        let query = TestQueryRepository::new(Arc::new(view_repo));
        let _ps = dynamodb_cqrs(client, vec![Box::new(query)], TestServices);
    }
}
