# cqrs-restful-demo

**A demo application using the
[cqrs-es2](https://github.com/brgirgis/cqrs-es2) framework.**

## Requirements

- rust stable
- docker and [docker-compose](https://docs.docker.com/compose/)
  for starting database instances
- [postman](https://www.postman.com/)
  (or curl or your favorite Restful client)

Alternatively, if a standard SQL database instance is running locally
it can be utilized instead of the docker instances,
see [the init script](db/init.sql) for the expected table
configuration.

## Installation

Clone this repository:

    git clone https://github.com/brgirgis/cqrs-restful-demo

Enter the project folder and start the docker stack:

    cd cqrs-restful-demo
    docker-compose up -d

Start the application

    cargo run

Call the API, the easiest way to do this is to import
the provided [postman collection](postman_collection.json)
into your Postman client.
Note that the command calls return a 204 status with no content.
For feedback on state you should call a query.
