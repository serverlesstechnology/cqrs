use crate::command_extractor::CommandExtractor;
use crate::state::ApplicationState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use cqrs_es::persist::ViewRepository;

// Serves as our query endpoint to respond with the materialized `BankAccountView`
// for the requested account.
pub async fn query_handler(
    Path(account_id): Path<String>,
    State(state): State<ApplicationState>,
) -> Response {
    let view = match state.account_query.load(&account_id).await {
        Ok(view) => view,
        Err(err) => {
            println!("Error: {:#?}\n", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response();
        }
    };
    match view {
        None => StatusCode::NOT_FOUND.into_response(),
        Some(account_view) => (StatusCode::OK, Json(account_view)).into_response(),
    }
}

// Serves as our command endpoint to make changes in a `BankAccount` aggregate.
pub async fn command_handler(
    Path(account_id): Path<String>,
    State(state): State<ApplicationState>,
    CommandExtractor(metadata, command): CommandExtractor,
) -> Response {
    match state
        .cqrs
        .execute_with_metadata(&account_id, command, metadata)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => {
            println!("Error: {:#?}\n", err);
            (StatusCode::BAD_REQUEST, err.to_string()).into_response()
        }
    }
}
