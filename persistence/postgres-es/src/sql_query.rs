pub(crate) struct SqlQueryFactory {
    event_table: String,
    select_events: String,
    insert_event: String,
    all_events: String,
    insert_snapshot: String,
    update_snapshot: String,
    select_snapshot: String,
}

impl SqlQueryFactory {
    pub fn new(event_table: &str, snapshot_table: &str) -> Self {
        Self {
            event_table: event_table.to_string(),
            select_events: format!("
SELECT aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata
  FROM {}
  WHERE aggregate_type = $1 AND aggregate_id = $2
  ORDER BY sequence", event_table),
            insert_event: format!("
INSERT INTO {} (aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata)
VALUES ($1, $2, $3, $4, $5, $6, $7)", event_table),
            all_events: format!("
SELECT aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata
  FROM {}
  WHERE aggregate_type = $1
  ORDER BY sequence", event_table),
            insert_snapshot: format!("
INSERT INTO {} (aggregate_type, aggregate_id, last_sequence, current_snapshot, payload)
VALUES ($1, $2, $3, $4, $5)", snapshot_table),
            update_snapshot: format!("
UPDATE {}
  SET last_sequence= $3 , payload= $6, current_snapshot= $4
  WHERE aggregate_type= $1 AND aggregate_id= $2 AND current_snapshot= $5", snapshot_table),
            select_snapshot: format!("
SELECT aggregate_type, aggregate_id, last_sequence, current_snapshot, payload
  FROM {}
  WHERE aggregate_type = $1 AND aggregate_id = $2", snapshot_table)
        }
    }
    pub fn select_events(&self) -> &str {
        &self.select_events
    }
    pub fn insert_event(&self) -> &str {
        &self.insert_event
    }
    pub fn insert_snapshot(&self) -> &str {
        &self.insert_snapshot
    }
    pub fn update_snapshot(&self) -> &str {
        &self.update_snapshot
    }
    pub fn select_snapshot(&self) -> &str {
        &self.select_snapshot
    }
    pub fn all_events(&self) -> &str {
        &self.all_events
    }
    pub fn get_last_events(&self, last_sequence: usize) -> String {
        format!(
            "
SELECT aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata
  FROM {}
  WHERE aggregate_type = $1 AND aggregate_id = $2 AND sequence > {}
  ORDER BY sequence",
            &self.event_table, last_sequence
        )
    }
}

#[test]
fn test_queries() {
    let query_factory = SqlQueryFactory::new("my_events", "my_snapshots");
    assert_eq!(
        query_factory.select_events(),
        "
SELECT aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata
  FROM my_events
  WHERE aggregate_type = $1 AND aggregate_id = $2
  ORDER BY sequence"
    );
    assert_eq!(query_factory.insert_event(), "
INSERT INTO my_events (aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata)
VALUES ($1, $2, $3, $4, $5, $6, $7)");
    assert_eq!(
        query_factory.all_events(),
        "
SELECT aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata
  FROM my_events
  WHERE aggregate_type = $1
  ORDER BY sequence"
    );
    assert_eq!(
        query_factory.insert_snapshot(),
        "
INSERT INTO my_snapshots (aggregate_type, aggregate_id, last_sequence, current_snapshot, payload)
VALUES ($1, $2, $3, $4, $5)"
    );
    assert_eq!(
        query_factory.update_snapshot(),
        "
UPDATE my_snapshots
  SET last_sequence= $3 , payload= $6, current_snapshot= $4
  WHERE aggregate_type= $1 AND aggregate_id= $2 AND current_snapshot= $5"
    );
    assert_eq!(
        query_factory.select_snapshot(),
        "
SELECT aggregate_type, aggregate_id, last_sequence, current_snapshot, payload
  FROM my_snapshots
  WHERE aggregate_type = $1 AND aggregate_id = $2"
    );
    assert_eq!(
        query_factory.get_last_events(20),
        "
SELECT aggregate_type, aggregate_id, sequence, event_type, event_version, payload, metadata
  FROM my_events
  WHERE aggregate_type = $1 AND aggregate_id = $2 AND sequence > 20
  ORDER BY sequence"
    );
}
