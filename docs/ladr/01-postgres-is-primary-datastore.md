# Postgres is the primary datastore

* Status: accepted
* Date: 2019-11-07

## Context

A backing datastore is needed to store events, snapshots and query views.
Though additional datastores may be supported in the future, the primary datastore must support all initial development 
requirements. As the reference datastore its eccentricities may also drive implementation decisions for new features.

## Decision Drivers

- Must be durable storage.
- Needs support for optimistic locking that is transactional across multiple tables.
- Will store serialized data of arbitrary sizes, generally in the low KB range but up to multiple MB for large views. 
- Suitable for serverless environments.
- Ideally available both as a platform and as a service to meet varying ops requirements.

## Considered Options

- PostgresSql
- MySql
- DynamoDb

## Decision Outcome

Postgres:
- Robust, ACID database with a large user base.
- Available as a serverless component via Amazon Aurora (along with MySql).