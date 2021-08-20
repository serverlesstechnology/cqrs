# cqrs-grpc-demo

**A demo application using the [cqrs-es2](https://github.com/brgirgis/cqrs-es2) framework.**

## Requirements

- rust stable
- docker and [docker-compose](https://docs.docker.com/compose/) for starting database instances

Alternatively, if a standard SQL database instance is running locally it can be utilized instead of the docker instances,
see [the init script](db/init.sql) for the expected table configuration.

## Installation

Clone this repository

    git clone https://github.com/brgirgis/cqrs-grpc-demo

Enter the project folder and start the docker stack:

    cd cqrs-grpc-demo
    docker-compose up -d

Start the application

    cargo run --bin server

Call the API using the provided client for testing the running application:

    cargo run --bin client
