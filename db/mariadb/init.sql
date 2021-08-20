CREATE DATABASE test;
USE test;

-- a single table is used for all events in the cqrs system
CREATE TABLE events
(
    aggregate_type VARCHAR(256)                 NOT NULL,
    aggregate_id   VARCHAR(256)                 NOT NULL,
    sequence       bigint CHECK (sequence >= 0)         ,
    payload        TEXT                                 ,
    metadata       TEXT                                 ,
    timestamp      timestamp DEFAULT (CURRENT_TIMESTAMP),
    PRIMARY KEY (aggregate_type, aggregate_id, sequence)
);

-- this table is only needed if snapshotting is employed
CREATE TABLE snapshots
(
    aggregate_type VARCHAR(256)                      NOT NULL,
    aggregate_id   VARCHAR(256)                      NOT NULL,
    last_sequence  bigint CHECK (last_sequence >= 0)         ,
    payload        TEXT                                      ,
    timestamp      timestamp DEFAULT (CURRENT_TIMESTAMP),
    PRIMARY KEY (aggregate_type, aggregate_id, last_sequence)
);

-- a single table is used for all queries in the cqrs system
CREATE TABLE queries
(
    aggregate_type VARCHAR(256)                NOT NULL,
    aggregate_id   VARCHAR(256)                NOT NULL,
    query_type     VARCHAR(256)                NOT NULL,
    version        bigint CHECK (version >= 0)         ,
    payload        TEXT                                ,
    PRIMARY KEY (aggregate_type, aggregate_id, query_type)
);

CREATE
    USER
    'test_user'@'%'
IDENTIFIED BY
    'test_pass';

GRANT
    ALL privileges
ON
    test.*
TO
    'test_user'@'%';

FLUSH PRIVILEGES;
