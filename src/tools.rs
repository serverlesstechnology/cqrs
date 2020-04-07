use std::error::Error;
use std::fmt::Debug;
use std::marker::PhantomData;

use postgres::Connection;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::aggregate::{Aggregate, AggregateError};
use crate::event::{DomainEvent, MessageEnvelope};
use crate::view::View;

pub struct GenericViewRepository<V, A, E>
    where V: View<A, E>,
          E: DomainEvent<A>,
          A: Aggregate
{
    view_name: String,
    error_handler: Option<Box<ErrorHandler>>,
    _phantom: PhantomData<(V, A, E)>,
}

type ErrorHandler = dyn Fn(AggregateError);

impl<V, A, E> GenericViewRepository<V, A, E>
    where V: View<A, E>,
          E: DomainEvent<A>,
          A: Aggregate
{
    pub fn new(view_name: String) -> Self {
        GenericViewRepository { view_name, error_handler: None, _phantom: PhantomData }
    }
    pub fn with_error_handler(&mut self, error_handler: Box<ErrorHandler>) {
        self.error_handler = Some(error_handler);
    }

    pub fn view_name(&self) -> String {
        self.view_name.to_string()
    }


    pub fn load_mut(&self, conn: &Connection, aggregate_id: String) -> Result<(V, ViewContext<V>), AggregateError> {
        let query = format!("SELECT version,payload FROM {} WHERE aggregate_id= $1", &self.view_name);
        let result = match conn.query(query.as_str(), &[&aggregate_id]) {
            Ok(result) => { result }
            Err(e) => {
                return Err(AggregateError::new(e.description()));
            }
        };
        match result.iter().next() {
            Some(row) => {
                let view_name = self.view_name.clone();
                let version = row.get("version");
                let payload = row.get("payload");
                let view = serde_json::from_value(payload)?;
                let view_context = ViewContext {
                    view_name,
                    aggregate_id,
                    version,
                    _phantom: PhantomData,
                };
                Ok((view, view_context))
            }
            None => {
                let view_context = ViewContext {
                    view_name: self.view_name.clone(),
                    aggregate_id,
                    version: 0,
                    _phantom: PhantomData,
                };
                Ok((Default::default(), view_context))
            }
        }
    }

    pub fn apply_events(&self, conn: &Connection, aggregate_id: &str, events: &[MessageEnvelope<A, E>])
    {
        match self.load_mut(conn, aggregate_id.to_string()) {
            Ok((mut view, view_context)) => {
                for event in events {
                    view.update(event);
                }
                view_context.commit(conn, view);
            }
            Err(e) => {
                match &self.error_handler {
                    None => {}
                    Some(handler) => {
                        (handler)(e);
                    }
                }
            }
        };
    }


    pub fn load(&self, conn: &Connection, aggregate_id: String) -> Option<V> {
        let query = format!("SELECT version,payload FROM {} WHERE aggregate_id= $1", &self.view_name);
        let result = conn.query(query.as_str(), &[&aggregate_id]).unwrap();
        match result.iter().next() {
            Some(row) => {
                let payload = row.get("payload");
                match serde_json::from_value(payload) {
                    Ok(view) => Some(view),
                    Err(e) => {
                        match &self.error_handler {
                            None => {}
                            Some(handler) => {
                                (handler)(e.into());
                            }
                        }
                        None
                    }
                }
            }
            None => None,
        }
    }
}


pub struct ViewContext<V>
    where V: Debug + Default + Serialize + DeserializeOwned + Default
{
    view_name: String,
    aggregate_id: String,
    version: i64,
    _phantom: PhantomData<V>,
}

impl<V> ViewContext<V>
    where V: Debug + Default + Serialize + DeserializeOwned + Default
{
    pub fn commit(&self, conn: &Connection, view: V) {
        let sql = match self.version {
            0 => format!("INSERT INTO {} (payload, version, aggregate_id) VALUES ( $1, $2, $3 )", &self.view_name),
            _ => format!("UPDATE {} SET payload= $1 , version= $2 WHERE aggregate_id= $3", &self.view_name),
        };
        let payload = serde_json::to_value(&view).unwrap();
        let version = self.version + 1;
        let aggregate_id = &self.aggregate_id;
        conn.execute(sql.as_str(), &[&payload, &version, aggregate_id]).unwrap();
    }
}
