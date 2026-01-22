use crate::{
    Event, Topic,
    testing::{EventEntry, EventHandle},
};

pub struct EventQuery<'a, E: Event, T: Topic<E>> {
    events: &'a [EventEntry<E, T>],
}

impl<'a, E: Event, T: Topic<E>> EventQuery<'a, E, T> {
    pub(crate) fn new(events: &'a [EventEntry<E, T>]) -> Self {
        Self { events }
    }

    pub fn count(&self) -> usize {
        self.events.len()
    }

    pub fn sent_by<N>(&self, actor: N) -> bool
    where
        N: for<'b> Into<&'b str>,
    {
        let actor = actor.into();
        self.events.iter().any(|e| e.sender_actor_eq(actor))
    }

    pub fn received_by<N>(&self, actor: N) -> bool
    where
        N: for<'b> Into<&'b str>,
    {
        let actor = actor.into();
        self.events.iter().any(|e| e.actor_eq(actor))
    }
}
