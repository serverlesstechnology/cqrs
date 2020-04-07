CREATE TABLE events (
    aggregate_type text NOT NULL,
    aggregate_id text NOT NULL,
    sequence bigint CHECK (sequence >= 0) NOT NULL,
    payload jsonb NOT NULL,
    metadata jsonb NOT NULL,
    timestamp timestamp with time zone DEFAULT (CURRENT_TIMESTAMP),
    PRIMARY KEY (aggregate_type, aggregate_id, sequence)
);