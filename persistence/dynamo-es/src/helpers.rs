use aws_sdk_dynamodb::client::Client;
use aws_sdk_dynamodb::operation::query::QueryOutput;
use aws_sdk_dynamodb::types::{AttributeValue, TransactWriteItem};
use serde_json::Value;
use std::collections::HashMap;

use crate::error::DynamoAggregateError;

pub(crate) async fn load_dynamo_view(
    client: &Client,
    table_name: &str,
    view_id: &str,
) -> Result<QueryOutput, DynamoAggregateError> {
    Ok(client
        .query()
        .table_name(table_name)
        .key_condition_expression("#view_type_id = :view_type_id")
        .expression_attribute_names("#view_type_id", "ViewId")
        .expression_attribute_values(":view_type_id", AttributeValue::S(String::from(view_id)))
        .send()
        .await?)
}

pub(crate) async fn commit_transactions(
    client: &Client,
    transactions: Vec<TransactWriteItem>,
) -> Result<(), DynamoAggregateError> {
    let transaction_len = transactions.len();
    if transaction_len > 25 {
        return Err(DynamoAggregateError::TransactionListTooLong(
            transaction_len,
        ));
    }
    client
        .transact_write_items()
        .set_transact_items(Some(transactions))
        .send()
        .await?;
    Ok(())
}

pub(crate) fn att_as_value(
    values: &HashMap<String, AttributeValue>,
    attribute_name: &str,
) -> Result<Value, DynamoAggregateError> {
    let attribute = require_attribute(values, attribute_name)?;
    match attribute.as_b() {
        Ok(payload_blob) => Ok(serde_json::from_slice(payload_blob.as_ref())?),
        Err(_) => Err(DynamoAggregateError::MissingAttribute(
            attribute_name.to_string(),
        )),
    }
}

pub(crate) fn att_as_number(
    values: &HashMap<String, AttributeValue>,
    attribute_name: &str,
) -> Result<usize, DynamoAggregateError> {
    let attribute = require_attribute(values, attribute_name)?;
    match attribute.as_n() {
        Ok(attribute_as_n) => match attribute_as_n.parse::<usize>() {
            Ok(attribute_number) => Ok(attribute_number),
            Err(_) => Err(DynamoAggregateError::MissingAttribute(
                attribute_name.to_string(),
            )),
        },
        Err(_) => Err(DynamoAggregateError::MissingAttribute(
            attribute_name.to_string(),
        )),
    }
}

pub(crate) fn att_as_string(
    values: &HashMap<String, AttributeValue>,
    attribute_name: &str,
) -> Result<String, DynamoAggregateError> {
    let attribute = require_attribute(values, attribute_name)?;
    match attribute.as_s() {
        Ok(attribute_as_s) => Ok(attribute_as_s.to_string()),
        Err(_) => Err(DynamoAggregateError::MissingAttribute(
            attribute_name.to_string(),
        )),
    }
}

pub(crate) fn require_attribute<'a>(
    values: &'a HashMap<String, AttributeValue>,
    attribute_name: &str,
) -> Result<&'a AttributeValue, DynamoAggregateError> {
    match values.get(attribute_name) {
        Some(attribute) => Ok(attribute),
        None => Err(DynamoAggregateError::MissingAttribute(
            attribute_name.to_string(),
        )),
    }
}
