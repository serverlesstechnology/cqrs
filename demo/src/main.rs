use axum::routing::get;
use axum::Router;
use cqrs_demo::route_handler::{command_handler, query_handler};
use cqrs_demo::state::new_application_state;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let state = new_application_state().await;
    // Configure the Axum routes and services.
    // For this example a single logical endpoint is used and the HTTP method
    // distinguishes whether the call is a command or a query.
    let router = Router::new()
        .route(
            "/account/:account_id",
            get(query_handler).post(command_handler),
        )
        .with_state(state);

    // Create the socket address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3030));

    // Start the Axum server
    println!("Server starting on {}", addr);
    axum::serve(
        tokio::net::TcpListener::bind(addr)
            .await
            .unwrap(),
        router,
    )
    .await
    .unwrap();
}