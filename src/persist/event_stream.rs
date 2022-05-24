use crate::persist::{PersistenceError, SerializedEvent};
use crate::{Aggregate, EventEnvelope};
use tokio::sync::mpsc::Receiver;

pub struct ReplayStream {
    queue: Receiver<Result<SerializedEvent, PersistenceError>>,
}

impl ReplayStream {
    pub fn new(queue: Receiver<Result<SerializedEvent, PersistenceError>>) -> Self {
        Self { queue }
    }

    pub async fn next<A: Aggregate>(
        &mut self,
    ) -> Option<Result<EventEnvelope<A>, PersistenceError>> {
        self.queue.recv().await.map(|result| match result {
            Ok(event) => match TryInto::try_into(event) {
                Ok(event) => Ok(event),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        })
    }
}
