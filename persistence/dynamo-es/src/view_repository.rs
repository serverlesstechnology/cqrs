use std::marker::PhantomData;

use async_trait::async_trait;
use aws_sdk_dynamodb::primitives::Blob;
use aws_sdk_dynamodb::types::{AttributeValue, Put, TransactWriteItem};
use cqrs_es::persist::{PersistenceError, ViewContext, ViewRepository};
use cqrs_es::{Aggregate, View};

use crate::helpers::{att_as_number, att_as_value, commit_transactions, load_dynamo_view};

/// A DynamoDb backed view repository for use in backing a `GenericQuery`.
pub struct DynamoViewRepository<V, A> {
    _phantom: PhantomData<(V, A)>,
    view_name: String,
    client: aws_sdk_dynamodb::client::Client,
}

impl<V, A> DynamoViewRepository<V, A>
where
    V: View<A>,
    A: Aggregate,
{
    /// Creates a new `DynamoViewRepository` that will store serialized views in a DynamoDb table named
    /// identically to the `view_name` value provided. This table should be created by the user
    /// before using this query repository (see `Makefile` for `create-table` command
    /// line example).
    ///
    /// ```
    /// # use cqrs_es::doc::MyAggregate;
    /// # use cqrs_es::persist::doc::MyView;
    /// use aws_sdk_dynamodb::Client;
    /// use dynamo_es::DynamoViewRepository;
    ///
    /// fn configure_view_repo(client: Client) -> DynamoViewRepository<MyView,MyAggregate> {
    ///     DynamoViewRepository::new("my_view_table", client)
    /// }
    /// ```
    pub fn new(view_name: &str, client: aws_sdk_dynamodb::client::Client) -> Self {
        Self {
            _phantom: Default::default(),
            view_name: view_name.to_string(),
            client,
        }
    }
}

#[async_trait]
impl<V, A> ViewRepository<V, A> for DynamoViewRepository<V, A>
where
    V: View<A>,
    A: Aggregate,
{
    async fn load(&self, view_id: &str) -> Result<Option<V>, PersistenceError> {
        let query_result = load_dynamo_view(&self.client, &self.view_name, view_id).await?;
        let query_items = match query_result.items {
            None => return Ok(None),
            Some(items) => items,
        };
        let query_item = match query_items.first() {
            None => return Ok(None),
            Some(item) => item,
        };
        let payload = att_as_value(query_item, "Payload")?;
        let view: V = serde_json::from_value(payload)?;
        Ok(Some(view))
    }

    async fn load_with_context(
        &self,
        view_id: &str,
    ) -> Result<Option<(V, ViewContext)>, PersistenceError> {
        let query_result = load_dynamo_view(&self.client, &self.view_name, view_id).await?;
        let query_items = match query_result.items {
            None => return Ok(None),
            Some(items) => items,
        };
        let query_item = match query_items.first() {
            None => {
                let view = V::default();
                let context = ViewContext::new(view_id.to_string(), 0);
                return Ok(Some((view, context)));
            }
            Some(item) => item,
        };
        let version = att_as_number(query_item, "ViewVersion")?;
        let payload = att_as_value(query_item, "Payload")?;
        let view: V = serde_json::from_value(payload)?;
        let context = ViewContext::new(view_id.to_string(), version as i64);
        Ok(Some((view, context)))
    }

    async fn update_view(&self, view: V, context: ViewContext) -> Result<(), PersistenceError> {
        let view_id = AttributeValue::S(String::from(&context.view_instance_id));
        let expected_view_version = AttributeValue::N(context.version.to_string());
        let view_version = AttributeValue::N((context.version + 1).to_string());
        let payload_blob = serde_json::to_vec(&view).unwrap();
        let payload = AttributeValue::B(Blob::new(payload_blob));
        let transaction = TransactWriteItem::builder()
            .put(Put::builder()
                .table_name(&self.view_name)
                .item("ViewId", view_id)
                .item("ViewVersion", view_version)
                .item("Payload", payload)
                .condition_expression("attribute_not_exists(ViewVersion) OR (ViewVersion  = :expected_view_version)")
                .expression_attribute_values(":expected_view_version", expected_view_version)
                .build()
                .map_err(|e|PersistenceError::UnknownError(Box::new(e)))?)
            .build();
        commit_transactions(&self.client, vec![transaction]).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use cqrs_es::persist::{ViewContext, ViewRepository};

    use crate::testing::tests::{
        test_dynamodb_client, Created, TestAggregate, TestEvent, TestView,
    };
    use crate::DynamoViewRepository;

    #[tokio::test]
    async fn test_valid_view_repository() {
        let repo = DynamoViewRepository::<TestView, TestAggregate>::new(
            "TestViewTable",
            test_dynamodb_client().await,
        );
        let test_view_id = uuid::Uuid::new_v4().to_string();

        let view = TestView {
            events: vec![TestEvent::Created(Created {
                id: "just a test event for this view".to_string(),
            })],
        };
        repo.update_view(view.clone(), ViewContext::new(test_view_id.to_string(), 0))
            .await
            .unwrap();
        let (found, context) = repo
            .load_with_context(&test_view_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found, view);

        let updated_view = TestView {
            events: vec![TestEvent::Created(Created {
                id: "a totally different view".to_string(),
            })],
        };
        repo.update_view(updated_view.clone(), context)
            .await
            .unwrap();
        let found_option = repo.load(&test_view_id).await.unwrap();
        let found = found_option.unwrap();

        assert_eq!(found, updated_view);
    }
}
