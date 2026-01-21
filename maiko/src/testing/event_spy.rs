use crate::{
    Event, EventId, Topic,
    testing::{EventEntry, EventHandle, spy_utils},
};

pub struct EventSpy<E: Event, T: Topic<E>> {
    id: EventId,
    data: Vec<EventEntry<E, T>>,
}

impl<E: Event, T: Topic<E>> EventSpy<E, T> {
    pub(crate) fn new(entries: &[EventEntry<E, T>], id: impl Into<EventId>) -> Self {
        let id = id.into();
        let data = spy_utils::filter_clone(entries, |e| id == e.event.id());
        Self { id, data }
    }

    pub fn was_delivered(&self) -> bool {
        !self.data.is_empty()
    }

    pub fn was_delivered_to<'a, N: Into<&'a str>>(&self, actor_name: N) -> bool {
        let actor_name = actor_name.into();
        self.data
            .iter()
            .any(|e| e.actor_name.as_ref() == actor_name)
    }

    pub fn receivers_count(&self) -> usize {
        spy_utils::receivers_count(&self.data)
    }

    pub fn receivers(&self) -> Vec<&str> {
        spy_utils::receivers(&self.data)
    }

    pub fn children(&self) -> Vec<EventHandle<E, T>> {
        self.data
            .iter()
            .filter(|e| {
                if let Some(cid) = e.event.meta().correlation_id() {
                    cid == self.id
                } else {
                    false
                }
            })
            .map(|e| EventHandle::from(e))
            .collect()
    }
}
