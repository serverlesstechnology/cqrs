# cqrs-demo

> A demo application using the [cqrs-es](https://github.com/serverlesstechnology/cqrs) framework
> with a backing postgres repository.

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

# Serverless cqrs-demo
A serverless demo is also available in this package.
The `bootstrap` binary that is built may be run on AWS Lambda but requires a number of services to do so (e.g., IAM roles, database, etc.). 
For simplicity this demo will only be deployed in docker and tested locally, and will use the same database as before.

## Additional Requirements
- The x86 MUSL library - get with `rustup target add x86_64-unknown-linux-musl`
- musl-gcc compiler - may be obtained on Ubuntu via `sudo apt install musl-tools`

## Building
Build a release version of the `bootstrap` binary targeting x86-MUSL and build this into a docker image using the provided Dockerfile.
```shell
cargo build --release \
  --target x86_64-unknown-linux-musl \
  --bin bootstrap
docker build -t cqrs-srvrls .
```

Ensure that the Postgres docker image is running, then start a new docker container using the created image.
```shell
docker run --rm --network=host cqrs-srvrls
```

Use the `test_lambda.sh` script in the `curl` directory to test the lambda container.
The application is designed to be deployed using an AWS API Gateway proxy integration or Lambda Function URL. 
These use the v2.0 of the AWS Lamba proxy integration, more information on 
[this format is available here](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-integrations-lambda.html#http-api-develop-integrations-lambda.proxy-format).
