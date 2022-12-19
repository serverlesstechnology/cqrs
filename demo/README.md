# cqrs-demo

> A demo application using the [cqrs-es](https://github.com/serverlesstechnology/cqrs) framework
> with a backing postgres repository.

Now using the [Axum server](https://crates.io/crates/axum-server) for a much simpler layout.

## Requirements
- rust 1.53 or greater
- docker & [docker-compose](https://docs.docker.com/compose/) for starting an instance of Postgres
- [postman](https://www.postman.com/) or [curl](curl/test_api.sh) (or your favorite Restful client)

Alternatively, if a standard Postgres instance is running locally it can be utilized instead of the docker instance,
see [the init script](db/init.sql) for the expected table configuration. 

## Installation

Clone this repository

    git clone https://github.com/serverlesstechnology/cqrs-demo

Enter the project folder and start a docker instance of PostgreSql

    cd cqrs-demo
    docker-compose up -d

Start the application

    cargo run

Call the API, the easiest way to do this is to import 
[the provided postman collection](cqrs-demo.postman_collection.json)
into your Postman client or the `test_api.sh` curl script found in the `curl` directory.
Note that the command calls are configured to return a 204 status with no content, 
only the query call will return a `200 OK` response with a body.
For feedback on state you should call a query.

### Docs you might want

- Documentation of these crates as well as an introduction to CQRS [can be found here](https://doc.rust-cqrs.org/).
- [Change log](https://github.com/serverlesstechnology/cqrs/blob/master/docs/versions/change_log.md) for the `cqrs-es` project at large.