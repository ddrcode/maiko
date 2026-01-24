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
        let receivers = EventQuery::new(records.clone()).received_by(actor.clone());
        let senders = EventQuery::new(records.clone()).sent_by(actor.clone());

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

    pub fn senders(&self) -> Vec<Arc<str>> {
        // spy_utils::distinct(&self.received_data, |e| e.event.meta().actor_name())
        todo!()
    }

    pub fn receivers(&self) -> Vec<Arc<str>> {
        todo!()
    }
}
