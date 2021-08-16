pub static INSERT_EVENT: &str = "
INSERT INTO
    events
    (
        aggregate_type, 
        aggregate_id,
        sequence,
        payload, 
        metadata
    )
VALUES
    (
        $1,
        $2,
        $3,
        $4,
        $5
    );
";

pub static SELECT_EVENTS: &str = "
SELECT
    sequence,
    payload
FROM
    events
WHERE
    aggregate_type = $1
    AND
    aggregate_id = $2
ORDER BY 
    sequence;
";

pub static SELECT_EVENTS_WITH_METADATA: &str = "
SELECT
    sequence,
    payload,
    metadata
FROM
    events
WHERE
    aggregate_type = $1
    AND
    aggregate_id = $2
ORDER BY 
    sequence;
";

pub static INSERT_SNAPSHOT: &str = "
INSERT INTO
    snapshots 
    (
        aggregate_type,
        aggregate_id, 
        last_sequence,
        payload
    )
VALUES
    (
        $1,
        $2,
        $3,
        $4
    );
";

pub static UPDATE_SNAPSHOT: &str = "
UPDATE
    snapshots
SET
    last_sequence = $3,
    payload = $4
WHERE
    aggregate_type = $1
    AND
    aggregate_id = $2;
";

pub static SELECT_SNAPSHOT: &str = "
SELECT
    last_sequence,
    payload
FROM
    snapshots
WHERE
    aggregate_type = $1
    AND
    aggregate_id = $2;
";

pub static INSERT_QUERY: &str = "
INSERT INTO
    queries 
    (
        aggregate_type,
        aggregate_id,
        query_type,
        version,
        payload
    )
VALUES
    (
        $1,
        $2,
        $3,
        $4,
        $5
    );
";

pub static UPDATE_QUERY: &str = "
UPDATE
    queries
SET
    version = $4,
    payload = $5
WHERE
    aggregate_type = $1
    AND
    aggregate_id = $2
    AND
    query_type = $3;
";

pub static SELECT_QUERY: &str = "
SELECT
    version,
    payload
FROM
    queries
WHERE
    aggregate_type = $1
    AND
    aggregate_id = $2
    AND
    query_type = $3;
";
