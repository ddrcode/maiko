use std::{collections::HashSet, sync::Arc};

use crate::{
    ActorHandle, Event, Topic, actor_handle,
    testing::{EventEntry, EventQuery, EventRecords, spy_utils},
};

pub struct ActorSpy<E: Event, T: Topic<E>> {
    actor: ActorHandle,
    records: EventRecords<E, T>,
    receivers: EventQuery<E, T>,
    senders: EventQuery<E, T>,
}

impl<E: Event, T: Topic<E>> ActorSpy<E, T> {
    pub(crate) fn new(records: EventRecords<E, T>, actor_handle: ActorHandle) -> Self {
        let actor = actor_handle;
        let receivers = EventQuery::new(records.clone()).received_by(&actor.clone());
        let senders = EventQuery::new(records.clone()).sent_by(&actor.clone());

        Self {
            actor,
            records,
            receivers,
            senders,
        }
    }

    pub fn received_events_count(&self) -> usize {
        self.receivers.count()
    }

    pub fn sent_events_count(&self) -> usize {
        spy_utils::distinct(self.senders.iter(), |e| e.event.id()).len()
    }

    pub fn receivers(&self) -> Vec<Arc<str>> {
        spy_utils::distinct(self.senders.iter(), |e| e.actor_name.clone())
    }

    pub fn senders(&self) -> Vec<Arc<str>> {
        spy_utils::distinct(self.receivers.iter(), |e| e.event.meta().actor_name.clone())
    }

    pub fn receivers_count(&self) -> usize {
        self.receivers().len()
    }

    pub fn senders_count(&self) -> usize {
        self.senders().len()
    }

    pub fn received_events(&self) -> EventQuery<E, T> {
        EventQuery::new(self.records.clone()).received_by(&self.actor.clone())
    }

    pub fn sent_events(&self) -> EventQuery<E, T> {
        EventQuery::new(self.records.clone()).sent_by(&self.actor.clone())
    }

    pub fn last_received(&self) -> Option<EventEntry<E, T>> {
        self.receivers.last()
    }

    pub fn last_sent(&self) -> Option<EventEntry<E, T>> {
        self.senders.last()
    }
}
