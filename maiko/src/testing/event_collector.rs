use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;

use crate::{ActorId, Envelope, Event, Topic, monitoring::Monitor, testing::EventEntry};

pub struct EventCollector<E: Event, T: Topic<E>> {
    events: UnboundedSender<EventEntry<E, T>>,
}

impl<E: Event, T: Topic<E>> EventCollector<E, T> {
    pub fn new(events: UnboundedSender<EventEntry<E, T>>) -> Self {
        Self { events }
    }
}

impl<E: Event, T: Topic<E>> Monitor<E, T> for EventCollector<E, T> {
    fn on_event_handled(&self, envelope: &Envelope<E>, topic: &T, receiver: &ActorId) {
        let event = Arc::new(envelope.clone());
        let topic = Arc::new(topic.clone());
        let actor_id = receiver.clone();
        let entry = EventEntry::new(event, topic, actor_id);
        let _ = self.events.send(entry);
    }
}
