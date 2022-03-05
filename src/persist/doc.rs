use crate::doc::MyAggregate;
use crate::persist::{
    PersistedEventRepository, PersistenceError, QueryContext, SerializedEvent, SerializedSnapshot,
    ViewRepository,
};
use crate::{Aggregate, EventEnvelope, View};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MyView;

impl View<MyAggregate> for MyView {
    fn update(&mut self, _event: &EventEnvelope<MyAggregate>) {
        todo!()
    }
}

pub struct MyDatabaseConnection;
pub struct MyViewRepository;

impl MyViewRepository {
    pub fn new(_db: MyDatabaseConnection) -> Self {
        Self
    }
}

#[async_trait]
impl ViewRepository<MyView, MyAggregate> for MyViewRepository {
    async fn load(
        &self,
        _view_id: &str,
    ) -> Result<Option<(MyView, QueryContext)>, PersistenceError> {
        todo!()
    }

    async fn update_view(
        &self,
        _view: MyView,
        _context: QueryContext,
    ) -> Result<(), PersistenceError> {
        todo!()
    }
}

pub struct MyEventRepository;

impl MyEventRepository {
    pub fn new(_db: MyDatabaseConnection) -> Self {
        Self
    }
}

#[async_trait]
impl PersistedEventRepository for MyEventRepository {
    async fn get_events<A: Aggregate>(
        &self,
        _aggregate_id: &str,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        todo!()
    }

    async fn get_last_events<A: Aggregate>(
        &self,
        _aggregate_id: &str,
        _number_events: usize,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        todo!()
    }

    async fn get_snapshot<A: Aggregate>(
        &self,
        _aggregate_id: &str,
    ) -> Result<Option<SerializedSnapshot>, PersistenceError> {
        todo!()
    }

    async fn persist<A: Aggregate>(
        &self,
        _events: &[SerializedEvent],
        _snapshot_update: Option<(String, Value, usize)>,
    ) -> Result<(), PersistenceError> {
        todo!()
    }
}
