# Change log

## `v0.6.1`

- Improve logging
- Improve unit test coverage

## `v0.6.0`

- Improve docs
- Rename `IDomainEvent` to `IEvent`
- Rename `IDomainCommand` to `ICommand`
- Cleanup generics dependence
- Introduce new interfaces:
  - `ICommandHandler`
  - `IEventHandler`
  - `IEventDispatcher`
  - `IEventConsumer`

## `v0.5.0`

- Add multi-store support
- Reorganize test framework module
- Improve unittest coverage
- Improve error return
- Add
  - `IQuery::query_type()`
  - `IQueryStore`
  - `QueryContext`
  - `memory_store::QueryStore`

## `v0.4.0`

- Move `AggregateContext` to the `aggregates` module
- Rename public traits to have the `I` notation (`IAggregate`,`IDomainEvent`, etc.)
- Convert `IAggregateContext` to `AggregateContext`

## `v0.3.0`

- Add `DomainCommand` trait
- Remove `EventEnvelope::aggregate_type` data member
- Add `Clone` to `IAggregate`

## `v0.2.5`

- Minor doc fixes

## `v0.2.4`

- Fix license documentation
- Upgrade dev dependencies

## `v0.2.3`

- Rename Github repo

## `v0.2.2`

- Automate Github deployment

## `v0.2.1`

- Minor doc correction

## `v0.2.0`

- Transfer of ownership
- Upgrade dependencies
- Add GitHub CI support
- Convert to a modular structure
- Correct mutability to match recent PostgresSQL changes

## `v0.1.0`

- Corrected to move all command and event logic into the aggregate.
