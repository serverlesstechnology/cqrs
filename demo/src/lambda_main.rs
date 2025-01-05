use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use cqrs_demo::command_extractor::CommandExtractor;
use cqrs_demo::route_handler::{command_handler, query_handler};
use cqrs_demo::state::{new_application_state, ApplicationState};
use lambda_http::{run, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let state = new_application_state().await;
    let routes = Router::new().route(
        "/account/:account_id",
        get(lambda_query_handler).post(lambda_command_handler),
    );
    let app = Router::new().merge(routes).with_state(state);
    run(app).await?;
    Ok(())
}
pub async fn lambda_query_handler(
    Path(account_id): Path<String>,
    State(state): State<ApplicationState>,
) -> Result<Response, (StatusCode, String)> {
    Ok(query_handler(Path(account_id), State(state)).await)
}
async fn lambda_command_handler(
    Path(account_id): Path<String>,
    State(state): State<ApplicationState>,
    CommandExtractor(metadata, command): CommandExtractor,
) -> Result<Response, (StatusCode, String)> {
    Ok(command_handler(
        Path(account_id),
        State(state),
        CommandExtractor(metadata, command),
    )
    .await)
}
