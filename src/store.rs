use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::RwLock;

use postgres::Connection;

use crate::aggregate::{Aggregate, AggregateId, AggregateError};
use crate::event::{DomainEvent, MessageEnvelope};

/// The abstract central source for loading past events and committing new events.
pub trait EventStore<I, A, E>
    where A: Aggregate,
          E: DomainEvent<A>
{
    /// Load all events for a particular `aggregate_id`
    fn load(&self, aggregate_id: &I) -> Vec<MessageEnvelope<A, E>>;
    /// Commit new events
    fn commit(&self, events: Vec<MessageEnvelope<A, E>>) -> Result<(), AggregateError>;
}

///  Simple memory store only useful for testing purposes
pub struct MemStore<I, A, E>
    where
        I: AggregateId<A>,
        A: Aggregate,
        E: DomainEvent<A>
{
    events: Rc<LockedMessageEnvelopeMap<A, E>>,
    _phantom: PhantomData<(I, A)>,
}

impl<I, A, E> Default for MemStore<I, A, E>
    where
        I: AggregateId<A>,
        A: Aggregate,
        E: DomainEvent<A>
{
    fn default() -> Self {
        MemStore {
            events: Default::default(),
            _phantom: PhantomData,
        }
    }
}

type LockedMessageEnvelopeMap<A, E> = RwLock<HashMap<String, Vec<MessageEnvelope<A, E>>>>;

impl<I, A, E> MemStore<I, A, E>
    where
        I: AggregateId<A>,
        A: Aggregate,
        E: DomainEvent<A>
{
    /// Creates a new event store with a shared event map.
    pub fn new_with_shared_events(events: Rc<LockedMessageEnvelopeMap<A, E>>) -> Self {
        MemStore {
            events,
            _phantom: PhantomData,
        }
    }
    fn load_commited_events(&self, aggregate_id: String) -> Vec<MessageEnvelope<A, E>> {
        let event_map = self.events.read().unwrap();
        let mut committed_events: Vec<MessageEnvelope<A, E>> = Vec::new();
        match event_map.get(aggregate_id.to_string().as_str()) {
            None => {}
            Some(events) => {
                for event in events {
                    committed_events.push(event.clone());
                }
            }
        };
        committed_events
    }
    fn aggregate_id(&self, events: &[MessageEnvelope<A, E>]) -> String {
        let &first_event = events.iter().peekable().peek().unwrap();
        first_event.aggregate_id.to_string()
    }
}

impl<I, A, E> EventStore<I, A, E> for MemStore<I, A, E>
    where
        I: AggregateId<A>,
        A: Aggregate,
        E: DomainEvent<A>
{
    fn load(&self, aggregate_id: &I) -> Vec<MessageEnvelope<A, E>>
    {
        let events = self.load_commited_events(aggregate_id.to_string());
        println!("loading: {} events", &events.len());
        events
    }

    fn commit(&self, events: Vec<MessageEnvelope<A, E>>) -> Result<(), AggregateError> {
        let aggregate_id = self.aggregate_id(&events);
        let mut new_events = self.load_commited_events(aggregate_id.to_string());
        for event in events {
            new_events.push(event.clone());
        }
        println!("storing: {} events", &new_events.len());
        let mut event_map = self.events.write().unwrap();
        event_map.insert(aggregate_id.to_string(), new_events);
        Ok(())
    }
}


/// Storage engine using an Postgres backing. This is the only persistent store currently
/// provided.
pub struct PostgresStore<I, A, E>
    where
        I: AggregateId<A>,
        A: Aggregate,
        E: DomainEvent<A>
{
    conn: Connection,
    _phantom: PhantomData<(I, A, E)>,
}

impl<I, A, E> PostgresStore<I, A, E>
    where
        I: AggregateId<A>,
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

impl<I, A, E> EventStore<I, A, E> for PostgresStore<I, A, E>
    where
        I: AggregateId<A>,
        A: Aggregate,
        E: DomainEvent<A>
{
    fn load(&self, aggregate_id: &I) -> Vec<MessageEnvelope<A, E>> {
        let agg_type = aggregate_id.aggregate_type();
        let id = aggregate_id.to_string();
        let mut result = Vec::new();
        match self.conn.query(SELECT_EVENTS, &[&agg_type, &id]) {
            Ok(rows) => {
                for row in rows.iter() {
                    let aggregate_type: String = row.get("aggregate_type");
                    let aggregate_id: String = row.get("aggregate_id");
                    let s: i64 = row.get("sequence");
                    let sequence = s as usize;
                    let payload: E = serde_json::from_value(row.get("payload")).unwrap();
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
            Ok(t) => {t},
            Err(err) => {
                return Err(AggregateError::TechnicalError(err.to_string()))
            },
        };
        for event in events {
            let agg_type = event.aggregate_type.clone();
            let id = event.aggregate_id.clone();
            let sequence = event.sequence as i64;
            let payload = serde_json::to_value(&event.payload).unwrap();
            let metadata = serde_json::to_value(&event.metadata).unwrap();
            self.conn.execute(INSERT_EVENT, &[&agg_type, &id, &sequence, &payload, &metadata]).unwrap();
        }
        match trans.commit() {
            Ok(_) => Ok(()),
            Err(err) => Err(AggregateError::TechnicalError(err.to_string())),
        }
    }
}