# cqrs-es2

**A lightweight, opinionated CQRS and event sourcing framework targeting serverless architectures.**

[![Publish](https://github.com/brgirgis/cqrs-es2/actions/workflows/crates-io.yml/badge.svg)](https://github.com/brgirgis/cqrs-es2/actions/workflows/crates-io.yml)
[![Test](https://github.com/brgirgis/cqrs-es2/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/brgirgis/cqrs-es2/actions/workflows/rust-ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cqrs-es2)](https://crates.io/crates/cqrs-es2)
[![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/cqrs-es2)

---

## Installation

```toml
[dependencies]
cqrs-es2 = "0.3.1"
serde = { version = "^1.0.127", features = ["derive"] }
serde_json = "^1.0.66"
```

## Usage

Documentation [is available here](https://doc.rust-cqrs.org) along with an introduction to CQRS and event sourcing.

Demo applications:

- [RESTful](https://github.com/brgirgis/cqrs-restful-demo).
- [gRPC](https://github.com/brgirgis/cqrs-grpc-demo).

## Change log

### `v0.3.1`

- Move `AggregateContext` to the `aggregates` module

### `v0.3.0`

- Add `DomainCommand` trait
- Remove `EventEnvelope::aggregate_type` data member

### `v0.2.5`

- Minor doc fixes

### `v0.2.4`

- Fix license documentation
- Upgrade dev dependencies

### `v0.2.3`

- Rename Github repo

### `v0.2.2`

- Automate Github deployment

### `v0.2.1`

- Minor doc correction

### `v0.2.0`

- Transfer of ownership
- Upgrade dependencies
- Add GitHub CI support
- Convert to a modular structure
- Correct mutability to match recent PostgresSQL changes

#### `v0.1.0`

- Corrected to move all command and event logic into the aggregate.

## TODO

- Event upcasters.
- Event serialization uses the event type as the root node of the JSON tree. This simplifies
  deserialization but is non-standard.
- A persistence implementation for DynamoDb.
- A persistence implementation for MySql.
