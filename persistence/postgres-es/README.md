# postgres-es

> A Postgres implementation of the `PersistedEventRepository` trait in cqrs-es.

---

## Usage
Add to your Cargo.toml file:

```toml
[dependencies]
cqrs-es = "0.4.11"
postgres-es = "0.4.11"
```

Requires access to a Postgres DB with existing tables. See:
- [Sample database configuration](db/init.sql)
- Use `docker-compose` to quickly setup [a local database](docker-compose.yml)

A simple configuration example:
```
let store = default_postgress_pool("postgresql://my_user:my_pass@localhost:5432/my_db");
let cqrs = postgres_es::postgres_cqrs(pool, vec![])
```

Things that could be helpful:
- [User guide](https://doc.rust-cqrs.org) along with an introduction to CQRS and event sourcing.
- [Demo application](https://github.com/serverlesstechnology/cqrs-demo) using the warp http server.
- [Change log](https://github.com/serverlesstechnology/cqrs/blob/main/docs/versions/change_log.md)

## Runtime and TLS configuration
This package defaults to expect the [Tokio runtime](https://crates.io/crates/tokio) and the
[Rustls library](https://crates.io/crates/rustls) for TLS.
If a different combination is desired the appropriate feature flag should be used:
- `runtime-tokio-native-tls`
- `runtime-tokio-rustls` (default)
- `runtime-async-std-native-tls`
- `runtime-async-std-rustls`
- `runtime-actix-native-tls`
- `runtime-actix-rustls`

[![Crates.io](https://img.shields.io/crates/v/postgres-es)](https://crates.io/crates/postgres-es)
[![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/postgres-es)
![docs](https://codebuild.us-west-2.amazonaws.com/badges?uuid=eyJlbmNyeXB0ZWREYXRhIjoiVVUyR0tRbTZmejFBYURoTHdpR3FnSUFqKzFVZE9JNW5haDZhcUFlY2xtREhtaVVJMWsxcWZOeC8zSUR0UWhpaWZMa0ZQSHlEYjg0N2FoU2lwV1FsTXFRPSIsIml2UGFyYW1ldGVyU3BlYyI6IldjUVMzVEpKN1V3aWxXWGUiLCJtYXRlcmlhbFNldFNlcmlhbCI6MX0%3D&branch=main)
