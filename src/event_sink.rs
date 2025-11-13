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

    pub async fn drain(&self) -> Vec<A::Event> {
        let mut events = self.events.lock().await;
        let result = events.clone();
        events.clear();
        result
    }
}
