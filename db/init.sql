-- a single table is used for all events in the cqrs system
CREATE TABLE events (
    aggregate_type text NOT NULL,
    aggregate_id text NOT NULL,
    sequence bigint CHECK (sequence >= 0) NOT NULL,
    payload jsonb NOT NULL,
    metadata jsonb NOT NULL,
    timestamp timestamp with time zone DEFAULT (CURRENT_TIMESTAMP),
    PRIMARY KEY (aggregate_type, aggregate_id, sequence)
);

-- one view table should be created for every `GenericViewRepository` used
-- replace name with the value used in `GenericViewRepository::new(view_name: String)`
CREATE TABLE xxxxxx_view (
    aggregate_id text                        NOT NULL,
    version      bigint CHECK (version >= 0) NOT NULL,
    payload      jsonb                       NOT NULL,
    PRIMARY KEY (aggregate_id)
);