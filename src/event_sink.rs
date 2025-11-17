use crate::Aggregate;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct EventSink<A: Aggregate> {
    events: Mutex<Vec<A::Event>>,
}

impl<A: Aggregate> Default for EventSink<A> {
    fn default() -> Self {
        Self {
            events: Default::default(),
        }
    }
}

impl<A: Aggregate> EventSink<A> {
    pub async fn write(&self, event: A::Event, aggregate: &mut A) {
        aggregate.apply(event.clone());
        self.events.lock().await.push(event);
    }

    pub async fn collect(self) -> Vec<A::Event> {
        self.events.into_inner()
    }
}
