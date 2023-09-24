use crate::persist::{PersistenceError, SerializedEvent};
use crate::{Aggregate, EventEnvelope};
use async_trait::async_trait;
use tokio::sync::mpsc::{Receiver, Sender};

/// Accesses a domain event stream for a particular aggregate.
///
#[async_trait::async_trait]
pub trait ReplayStream: Send + Sync {
    /// Receive the next event or error in the stream.
    /// If no event is available this should block.
    async fn next<A: Aggregate>(&mut self) -> Option<Result<EventEnvelope<A>, PersistenceError>>;
}

/// An implementation of `ReplayStream` that uses a `tokio::sync::mpsc::Receiver` and `Sender`
/// to stream events between threads.
///
/// _Note: design expected to change after [implemention of RFC 2996](https://github.com/rust-lang/rust/issues/79024)._
pub struct MpscReplayStream {
    queue: Receiver<Result<SerializedEvent, PersistenceError>>,
}

#[async_trait]
impl ReplayStream for MpscReplayStream {
    /// Receive the next event or error in the stream, if no event is available this will block.
    async fn next<A: Aggregate>(&mut self) -> Option<Result<EventEnvelope<A>, PersistenceError>> {
        self.queue
            .recv()
            .await
            .map(|result| result.and_then(TryInto::try_into))
    }
}
impl MpscReplayStream {
    /// Creates a new `ReplayStream` that will buffer events up to the `queue_size`.
    pub fn new(queue_size: usize) -> (ReplayFeed, Self) {
        let (sender, queue) = tokio::sync::mpsc::channel(queue_size);
        (ReplayFeed { sender }, Self { queue })
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
        next_event: Result<SerializedEvent, PersistenceError>,
    ) -> Result<(), PersistenceError> {
        self.sender.send(next_event).await?;
        Ok(())
    }
}
#[cfg(test)]
mod test {
    use crate::doc::MyAggregate;
    use crate::persist::event_stream::ReplayStream;
    use crate::persist::{MpscReplayStream, PersistenceError};
    #[tokio::test]
    async fn test_replay_stream() {
        let (mut feed, mut stream) = MpscReplayStream::new(5);
        feed.push(Err(PersistenceError::OptimisticLockError))
            .await
            .unwrap();
        drop(feed);
        let found = stream.next::<MyAggregate>().await;
        assert!(
            matches!(
                found.unwrap().unwrap_err(),
                PersistenceError::OptimisticLockError
            ),
            "expected optimistic lock error"
        );
    }
}
