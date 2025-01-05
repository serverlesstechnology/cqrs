-- a single table is used for all events in the cqrs system
CREATE TABLE events
(
    aggregate_type varchar(255)                 NOT NULL,
    aggregate_id   varchar(255)                 NOT NULL,
    sequence       bigint CHECK (sequence >= 0) NOT NULL,
    event_type     text                         NOT NULL,
    event_version  text                         NOT NULL,
    payload        json                         NOT NULL,
    metadata       json                         NOT NULL,
    CONSTRAINT events_pk PRIMARY KEY (aggregate_type, aggregate_id, sequence)
);

-- this table is only needed if snapshotting is employed
CREATE TABLE snapshots
(
    aggregate_type   varchar(255)                         NOT NULL,
    aggregate_id     varchar(255)                         NOT NULL,
    last_sequence    bigint CHECK (last_sequence >= 0)    NOT NULL,
    current_snapshot bigint CHECK (current_snapshot >= 0) NOT NULL,
    payload          json                                 NOT NULL,
    CONSTRAINT snapshots_pk PRIMARY KEY (aggregate_type, aggregate_id)
);

-- one view table should be created for every `MysqlViewRepository` used
-- replace name with the value used in `MysqlViewRepository::new(view_name: String)`
CREATE TABLE test_view
(
    view_id varchar(255)                NOT NULL,
    version bigint CHECK (version >= 0) NOT NULL,
    payload json                        NOT NULL,
    CONSTRAINT test_view_pk PRIMARY KEY (view_id)
);

INSERT INTO events (aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata)
VALUES ('Customer', 'previous_event_in_need_of_upcast', 1, 'NameAdded', '1.0', '{
  "NameAdded": {}
}', '{}');
