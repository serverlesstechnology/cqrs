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
cqrs-es = "0.1.0"
```

## Usage

Documentation [is available here](https://doc.rust-cqrs.org) along with an introduction to CQRS and event sourcing.

A demo application [is available here](https://github.com/serverlesstechnology/cqrs-demo).

## Change log

#### `v0.1.0` 
Corrected to move all command and event logic into the aggregate. 

## Todos

- Event upcasters.
- Event serialization uses the event type as the root node of the JSON tree. This simplifies
deserialization but is non-standard.
- A persistence implementation for DynamoDb.
- A persistence implementation for MySql.