# cqrs

**A lightweight, opinionated CQRS and event sourcing framework targeting serverless architectures.**

![Build tag](https://codebuild.us-west-2.amazonaws.com/badges?uuid=eyJlbmNyeXB0ZWREYXRhIjoia3ZYcXozMjVZaFhoTldlUmhHemlWVm9LUjVaTC9LN3dSTFZpMkVTTmRycElkcGhJT3g2TUdtajZyRWZMd01xNktvUkNwLzdZYW15bzJkZldQMjJWZ1dNPSIsIml2UGFyYW1ldGVyU3BlYyI6InFORDNyaFFEQUNFQkE1NlUiLCJtYXRlcmlhbFNldFNlcmlhbCI6MX0%3D&branch=master)
[![Crates.io](https://img.shields.io/crates/v/cqrs-es)](https://crates.io/crates/cqrs-es)
[![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/cqrs-es)
---

## Installation

cqrs-es is available from Crates.io or Github.

```toml
[dependencies]
cqrs-es = "0.0.16"
```

## Opinions

- Aggregate persistence is via event sourcing only.
- Support for JSON serialization only.
- Generics are preferred over boxed traits.
- Persistence is implemented through a Postgres database.

## Todos/research

- Event upcasters.
- Event serialization uses the event type as the root node of the JSON tree. This simplifies
deserialization but is non-standard.
- A persistence implementation for DynamoDb.
- Support for snapshots.