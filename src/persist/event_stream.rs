use crate::persist::{EventUpcaster, PersistenceError, SerializedEvent};
use crate::{Aggregate, EventEnvelope};
use tokio::sync::mpsc::{Receiver, Sender};

/// Accesses a domain event stream for a particular aggregate.
///
/// _Note: design expected to change after [implementation of RFC 2996](https://github.com/rust-lang/rust/issues/79024)._
pub struct ReplayStream {
    queue: Receiver<Result<SerializedEvent, PersistenceError>>,
}

impl ReplayStream {
    /// Creates a new `ReplayStream` that will buffer events up to the `queue_size`.
    pub fn new(queue_size: usize) -> (ReplayFeed, Self) {
        let (sender, queue) = tokio::sync::mpsc::channel(queue_size);
        (ReplayFeed { sender }, Self { queue })
    }

    /// Receive the next upcasted event or error in the stream, if no event is available this will block.
    pub async fn next<A: Aggregate>(
        &mut self,
        upcasters: &[Box<dyn EventUpcaster>],
    ) -> Option<Result<EventEnvelope<A>, PersistenceError>> {
        self.queue
            .recv()
            .await
            .map(|result| result.and_then(|event| event.upcast(upcasters).try_into()))
    }
}

/// Used to send events to a `ReplayStream` for replaying events.
pub struct ReplayFeed {
    sender: Sender<Result<SerializedEvent, PersistenceError>>,
}

impl ReplayFeed {
    /// Push the next event onto the stream.
    pub async fn push(
        &mut self,
        result: Result<SerializedEvent, PersistenceError>,
    ) -> Result<(), PersistenceError> {
        self.sender.send(result).await?;
        Ok(())
    }
}
#[cfg(test)]
mod test {
    use crate::doc::MyAggregate;
    use crate::persist::{PersistenceError, ReplayStream};

    #[tokio::test]
    async fn test_replay_stream() {
        let (mut feed, mut stream) = ReplayStream::new(5);
        feed.push(Err(PersistenceError::OptimisticLockError))
            .await
            .unwrap();
        drop(feed);
        let found = stream.next::<MyAggregate>(&[]).await;
        assert!(
            matches!(
                found.unwrap().unwrap_err(),
                PersistenceError::OptimisticLockError
            ),
            "expected optimistic lock error"
        );
    }
}
