use std::collections::HashSet;

use crate::{
    Event, Topic,
    testing::{EventEntry, spy_utils},
};

pub struct ActorSpy<E: Event, T: Topic<E>> {
    received_data: Vec<EventEntry<E, T>>,
    sent_data: Vec<EventEntry<E, T>>,
}

impl<E: Event, T: Topic<E>> ActorSpy<E, T> {
    pub(crate) fn new<'a, N>(entries: &[EventEntry<E, T>], actor_name: N) -> Self
    where
        N: Into<&'a str>,
    {
        let actor_name = actor_name.into();
        let received_data =
            spy_utils::filter_clone(entries, |e| e.actor_name.as_ref() == actor_name);
        let sent_data =
            spy_utils::filter_clone(entries, |e| e.event.meta().actor_name() == actor_name);

        Self {
            received_data,
            sent_data,
        }
    }

    pub fn received_events_count(&self) -> usize {
        self.received_data.len()
    }

    pub fn sent_events_count(&self) -> usize {
        self.sent_data
            .iter()
            .map(|data| data.event.id())
            .collect::<HashSet<_>>()
            .len()
    }

    pub fn senders(&self) -> Vec<&str> {
        // spy_utils::distinct(&self.received_data, |e| e.event.meta().actor_name())
        todo!()
    }

    pub fn receivers(&self) -> Vec<&str> {
        spy_utils::receivers(&self.sent_data)
    }
}
