use std::marker::PhantomData;
use postgres::Connection;

use crate::aggregate::{Aggregate, AggregateError};
use crate::event::{DomainEvent, MessageEnvelope};

/// The abstract central source for loading past events and committing new events.
pub trait EventStore<A, E>
    where A: Aggregate,
          E: DomainEvent<A>
{
    /// Load all events for a particular `aggregate_id`
    fn load(&self, aggregate_id: &str) -> Vec<MessageEnvelope<A, E>>;
    /// Commit new events
    fn commit(&self, events: Vec<MessageEnvelope<A, E>>) -> Result<(), AggregateError>;
}

/// Storage engine using an Postgres backing. This is the only persistent store currently
/// provided.
pub struct PostgresStore<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    conn: Connection,
    _phantom: PhantomData<(A, E)>,
}

impl<A, E> PostgresStore<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    /// Creates a new `PostgresStore` from the provided database connection.
    pub fn new(conn: Connection) -> Self {
        PostgresStore {
            conn,
            _phantom: PhantomData,
        }
    }
}

static INSERT_EVENT: &str = "INSERT INTO events (aggregate_type, aggregate_id, sequence, payload, metadata)
                               VALUES ($1, $2, $3, $4, $5)";
static SELECT_EVENTS: &str = "SELECT aggregate_type, aggregate_id, sequence, payload, metadata
                                FROM events
                                WHERE aggregate_type = $1 AND aggregate_id = $2 ORDER BY sequence";

impl<A, E> EventStore<A, E> for PostgresStore<A, E>
    where
        A: Aggregate,
        E: DomainEvent<A>
{
    fn load(&self, aggregate_id: &str) -> Vec<MessageEnvelope<A, E>> {
        let agg_type = A::aggregate_type();
        let id = aggregate_id.to_string();
        let mut result = Vec::new();
        match self.conn.query(SELECT_EVENTS, &[&agg_type, &id]) {
            Ok(rows) => {
                for row in rows.iter() {
                    let aggregate_type: String = row.get("aggregate_type");
                    let aggregate_id: String = row.get("aggregate_id");
                    let s: i64 = row.get("sequence");
                    let sequence = s as usize;
                    let payload: E = match serde_json::from_value(row.get("payload")) {
                        Ok(payload) => payload,
                        Err(err) => {
                            panic!("bad payload found in events table for aggregate id {} with error: {}", &id, err);
                        }
                    };
                    let event = MessageEnvelope {
                        aggregate_id,
                        sequence,
                        aggregate_type,
                        payload,
                        metadata: Default::default(),
                        _phantom: PhantomData,
                    };
                    result.push(event);
                }
            }
            Err(e) => { println!("{:?}", e); }
        }
        result
    }

    fn commit(&self, events: Vec<MessageEnvelope<A, E>>) -> Result<(), AggregateError> {
        let trans = match self.conn.transaction() {
            Ok(t) => { t }
            Err(err) => {
                return Err(AggregateError::TechnicalError(err.to_string()));
            }
        };
        for event in events {
            let agg_type = event.aggregate_type.clone();
            let id = event.aggregate_id.clone();
            let sequence = event.sequence as i64;
            let payload = match serde_json::to_value(&event.payload) {
                Ok(payload) => payload,
                Err(err) => {
                    panic!("bad payload found in events table for aggregate id {} with error: {}", &id, err);
                }
            };
            let metadata = match serde_json::to_value(&event.metadata) {
                Ok(metadata) => metadata,
                Err(err) => {
                    panic!("bad metadata found in events table for aggregate id {} with error: {}", &id, err);
                }
            };
            match self.conn.execute(INSERT_EVENT, &[&agg_type, &id, &sequence, &payload, &metadata]) {
                Ok(_) => {}
                Err(err) => {
                    panic!("unable to insert event table for aggregate id {} with error: {}\n  and payload: {}", &id, err, &payload);
                }
            };
        }
        match trans.commit() {
            Ok(_) => Ok(()),
            Err(err) => Err(AggregateError::TechnicalError(err.to_string())),
        }
    }
}