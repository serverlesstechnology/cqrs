# cqrs-es2

**A lightweight, opinionated CQRS and event sourcing framework targeting serverless architectures.**

[![Publish](https://github.com/brgirgis/cqrs-es2/actions/workflows/crates-io.yml/badge.svg)](https://github.com/brgirgis/cqrs-es2/actions/workflows/crates-io.yml)
[![Test](https://github.com/brgirgis/cqrs-es2/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/brgirgis/cqrs-es2/actions/workflows/rust-ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cqrs-es2)](https://crates.io/crates/cqrs-es2)
[![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/cqrs-es2)

---

## Installation

cqrs-es2 is available from Crates.io or Github.

```toml
[dependencies]
cqrs-es2 = "0.2.3"
```

## Usage

Documentation [is available here](https://doc.rust-cqrs.org) along with an introduction to CQRS and event sourcing.

A demo application [is available here](https://github.com/brgirgis/cqrs-es2-demo).

## Change log

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

## Todos

- Event upcasters.
- Event serialization uses the event type as the root node of the JSON tree. This simplifies
  deserialization but is non-standard.
- A persistence implementation for DynamoDb.
- A persistence implementation for MySql.
