use sqlx::{AssertSqlSafe, SqlSafeStr, SqlStr};

pub(crate) struct SqlQueryFactory {
    event_table: &'static str,
    select_events: SqlStr,
    insert_event: SqlStr,
    all_events: SqlStr,
    insert_snapshot: SqlStr,
    update_snapshot: SqlStr,
    select_snapshot: SqlStr,
}

impl SqlQueryFactory {
    pub fn new(event_table: &'static str, snapshot_table: &'static str) -> Self {
        Self {
            event_table: event_table,
            select_events: AssertSqlSafe(format!("
SELECT aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata
  FROM {event_table}
  WHERE aggregate_type = $1 AND aggregate_id = $2
  ORDER BY sequence")).into_sql_str(),
            insert_event: AssertSqlSafe(format!("
INSERT INTO {event_table} (aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata)
VALUES ($1, $2, $3, $4, $5, $6, $7)")).into_sql_str(),
            all_events: AssertSqlSafe(format!("
SELECT aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata
  FROM {event_table}
  WHERE aggregate_type = $1
  ORDER BY sequence")).into_sql_str(),
            insert_snapshot: AssertSqlSafe(format!("
INSERT INTO {snapshot_table} (aggregate_type, aggregate_id, last_sequence, current_snapshot, payload)
VALUES ($1, $2, $3, $4, $5)")).into_sql_str(),
            update_snapshot: AssertSqlSafe(format!("
UPDATE {snapshot_table}
  SET last_sequence= $3 , payload= $6, current_snapshot= $4
  WHERE aggregate_type= $1 AND aggregate_id= $2 AND current_snapshot= $5")).into_sql_str(),
            select_snapshot: AssertSqlSafe(format!("
SELECT aggregate_type, aggregate_id, last_sequence, current_snapshot, payload
  FROM {snapshot_table}
  WHERE aggregate_type = $1 AND aggregate_id = $2")).into_sql_str(),
        }
    }
    pub fn select_events(&self) -> SqlStr {
        self.select_events.clone()
    }
    pub fn insert_event(&self) -> SqlStr {
        self.insert_event.clone()
    }
    pub fn insert_snapshot(&self) -> SqlStr {
        self.insert_snapshot.clone()
    }
    pub fn update_snapshot(&self) -> SqlStr {
        self.update_snapshot.clone()
    }
    pub fn select_snapshot(&self) -> SqlStr {
        self.select_snapshot.clone()
    }
    pub fn all_events(&self) -> SqlStr {
        self.all_events.clone()
    }
    pub fn get_last_events(&self, last_sequence: usize) -> SqlStr {
        AssertSqlSafe(format!(
            "
SELECT aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata
  FROM {}
  WHERE aggregate_type = $1 AND aggregate_id = $2 AND sequence > {}
  ORDER BY sequence",
            self.event_table, last_sequence
        ))
        .into_sql_str()
    }
}

#[test]
fn test_queries() {
    let query_factory = SqlQueryFactory::new("my_events", "my_snapshots");
    assert_eq!(
        query_factory.select_events().as_str(),
        "
SELECT aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata
  FROM my_events
  WHERE aggregate_type = $1 AND aggregate_id = $2
  ORDER BY sequence"
    );
    assert_eq!(query_factory.insert_event().as_str(), "
INSERT INTO my_events (aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata)
VALUES ($1, $2, $3, $4, $5, $6, $7)");
    assert_eq!(
        query_factory.all_events().as_str(),
        "
SELECT aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata
  FROM my_events
  WHERE aggregate_type = $1
  ORDER BY sequence"
    );
    assert_eq!(
        query_factory.insert_snapshot().as_str(),
        "
INSERT INTO my_snapshots (aggregate_type, aggregate_id, last_sequence, current_snapshot, payload)
VALUES ($1, $2, $3, $4, $5)"
    );
    assert_eq!(
        query_factory.update_snapshot().as_str(),
        "
UPDATE my_snapshots
  SET last_sequence= $3 , payload= $6, current_snapshot= $4
  WHERE aggregate_type= $1 AND aggregate_id= $2 AND current_snapshot= $5"
    );
    assert_eq!(
        query_factory.select_snapshot().as_str(),
        "
SELECT aggregate_type, aggregate_id, last_sequence, current_snapshot, payload
  FROM my_snapshots
  WHERE aggregate_type = $1 AND aggregate_id = $2"
    );
    assert_eq!(
        query_factory.get_last_events(20).as_str(),
        "
SELECT aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata
  FROM my_events
  WHERE aggregate_type = $1 AND aggregate_id = $2 AND sequence > 20
  ORDER BY sequence"
    );
}
