# Change log

#### `v0.2.4`
- Move to Rust 2021 edition.
- Audit and update dependencies.

#### `v0.2.1`
- Moved generic persistence logic in from postgres-es package.
- Added event context information to event envelope.

#### `v0.2.0`
Moved to async/await for better tool support.

#### `v0.1.3`
Aggregates now consume events on `apply`.

#### `v0.1.2`
Require `Send + Sync` for queries.

#### `v0.1.1`
Require `Send + Sync` for support of multi-threaded applications.

#### `v0.1.0`
Corrected to move all command and event logic into the aggregate.
