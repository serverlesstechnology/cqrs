CREATE DATABASE test;
\c test;

-- a single table is used for all events in the cqrs system
CREATE TABLE events
(
    aggregate_type text                         NOT NULL,
    aggregate_id   text                         NOT NULL,
    sequence       bigint CHECK (sequence >= 0) NOT NULL,
    payload        jsonb                        NOT NULL,
    metadata       jsonb                        NOT NULL,
    timestamp      timestamp with time zone DEFAULT (CURRENT_TIMESTAMP),
    PRIMARY KEY (aggregate_type, aggregate_id, sequence)
);

-- this table is only needed if snapshotting is employed
CREATE TABLE snapshots
(
    aggregate_type text                              NOT NULL,
    aggregate_id   text                              NOT NULL,
    last_sequence  bigint CHECK (last_sequence >= 0) NOT NULL,
    payload        jsonb                             NOT NULL,
    timestamp      timestamp with time zone DEFAULT (CURRENT_TIMESTAMP),
    PRIMARY KEY (aggregate_type, aggregate_id, last_sequence)
);

-- a single table is used for all queries in the cqrs system
CREATE TABLE queries
(
    aggregate_type text                        NOT NULL,
    aggregate_id   text                        NOT NULL,
    query_type     text                        NOT NULL,
    version        bigint CHECK (version >= 0) NOT NULL,
    payload        jsonb                       NOT NULL,
    PRIMARY KEY (aggregate_type, aggregate_id, query_type)
);

CREATE
    USER
    test_user
WITH
    NOCREATEDB
ENCRYPTED PASSWORD
    'test_pass';

GRANT
    ALL PRIVILEGES
ON TABLE
    events,
    snapshots,
    queries
TO
    test_user;
