use crate::{
    Event, Topic,
    testing::{EventEntry, EventQuery, spy_utils},
};

pub struct TopicSpy<E: Event, T: Topic<E>> {
    data: Vec<EventEntry<E, T>>,
}

impl<E: Event, T: Topic<E>> TopicSpy<E, T> {
    pub(crate) fn new(entries: &[EventEntry<E, T>], topic: &T) -> Self {
        let data = spy_utils::filter_clone(entries, |e| &e.topic == topic);
        Self { data }
    }

    pub fn was_published(&self) -> bool {
        !self.data.is_empty()
    }

    pub fn event_count(&self) -> usize {
        self.data.len()
    }

    pub fn subscribers_count(&self) -> usize {
        todo!()
    }

    pub fn receivers(&self) -> Vec<&str> {
        spy_utils::distinct(&self.data, |e| e.actor_name.as_ref())
    }

    pub fn events<'a>(&'a self) -> EventQuery<'a, E, T> {
        EventQuery::new(&self.data)
    }
}
