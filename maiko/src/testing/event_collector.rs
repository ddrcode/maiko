use std::{cell::RefCell, collections::HashMap, sync::Arc};

use tokio::sync::mpsc::UnboundedSender;

use crate::{ActorId, Envelope, Event, EventId, Topic, monitoring::Monitor, testing::EventEntry};

pub struct EventCollector<E: Event, T: Topic<E>> {
    events: UnboundedSender<EventEntry<E, T>>,
    event_topics: RefCell<HashMap<EventId, (usize, Arc<T>)>>,
}

impl<E: Event, T: Topic<E>> EventCollector<E, T> {
    pub fn new(events: UnboundedSender<EventEntry<E, T>>) -> Self {
        Self {
            events,
            event_topics: RefCell::new(HashMap::new()),
        }
    }
}

impl<E: Event, T: Topic<E>> Monitor<E, T> for EventCollector<E, T> {
    fn on_event_dispatched(&self, envelope: &Envelope<E>, topic: &T, _receiver: &ActorId) {
        let mut topics = self.event_topics.borrow_mut();
        topics
            .entry(envelope.id())
            .and_modify(|e| e.0 += 1)
            .or_insert_with(|| (1, Arc::new(topic.clone())));
    }

    fn on_event_handled(&self, envelope: &Envelope<E>, actor_id: &ActorId) {
        let mut topics = self.event_topics.borrow_mut();
        let event_id = &envelope.id();
        if let Some(e) = topics.get_mut(event_id) {
            let topic = e.1.clone();
            if e.0 <= 1 {
                topics.remove(event_id);
            } else {
                e.0 -= 1;
            }
            let entry = EventEntry::new(Arc::new(envelope.clone()), topic, actor_id.clone());
            let _ = self.events.send(entry);
        }
    }
}
