use std::sync::Arc;

use crate::{
    ActorHandle, Event, Topic,
    testing::{EventEntry, EventQuery, EventRecords},
};

/// A spy for observing events from the perspective of a specific actor.
///
/// Provides methods to inspect:
/// - Events received by this actor (inbound)
/// - Events sent by this actor (outbound)
/// - Which actors this actor communicated with
pub struct ActorSpy<E: Event, T: Topic<E>> {
    #[allow(dead_code)]
    actor: ActorHandle,
    inbound: EventQuery<E, T>,
    outbound: EventQuery<E, T>,
}

impl<E: Event, T: Topic<E>> ActorSpy<E, T> {
    pub(crate) fn new(records: EventRecords<E, T>, actor: ActorHandle) -> Self {
        let inbound = EventQuery::new(records.clone()).received_by(&actor);
        let outbound = EventQuery::new(records).sent_by(&actor);

        Self {
            actor,
            inbound,
            outbound,
        }
    }

    // ==================== Inbound (events received by this actor) ====================

    /// Returns a query for events received by this actor.
    ///
    /// Use this to further filter or inspect inbound events.
    pub fn inbound(&self) -> EventQuery<E, T> {
        self.inbound.clone()
    }

    /// Returns the number of events received by this actor.
    pub fn inbound_count(&self) -> usize {
        self.inbound.count()
    }

    /// Returns the last event received by this actor.
    pub fn last_received(&self) -> Option<EventEntry<E, T>> {
        self.inbound.last()
    }

    /// Returns the names of actors that sent events to this actor.
    pub fn received_from(&self) -> Vec<Arc<str>> {
        distinct_by(&self.inbound, |e| e.meta().actor_name.clone())
    }

    /// Returns the count of distinct actors that sent events to this actor.
    pub fn received_from_count(&self) -> usize {
        self.received_from().len()
    }

    // ==================== Outbound (events sent by this actor) ====================

    /// Returns a query for events sent by this actor.
    ///
    /// Use this to further filter or inspect outbound events.
    pub fn outbound(&self) -> EventQuery<E, T> {
        self.outbound.clone()
    }

    /// Returns the number of distinct events sent by this actor.
    ///
    /// Note: This counts unique event IDs, not deliveries. A single event
    /// delivered to multiple actors counts as one.
    pub fn outbound_count(&self) -> usize {
        distinct_by(&self.outbound, |e| e.id()).len()
    }

    /// Returns the last event sent by this actor.
    pub fn last_sent(&self) -> Option<EventEntry<E, T>> {
        self.outbound.last()
    }

    /// Returns the names of actors that received events from this actor.
    pub fn sent_to(&self) -> Vec<Arc<str>> {
        distinct_by(&self.outbound, |e| e.actor_name.clone())
    }

    /// Returns the count of distinct actors that received events from this actor.
    pub fn sent_to_count(&self) -> usize {
        self.sent_to().len()
    }
}

/// Helper to collect distinct values from a query using a mapper function.
fn distinct_by<E, T, R, F>(query: &EventQuery<E, T>, mapper: F) -> Vec<R>
where
    E: Event,
    T: Topic<E>,
    R: std::hash::Hash + std::cmp::Eq,
    F: Fn(&EventEntry<E, T>) -> R,
{
    use std::collections::HashSet;
    query
        .iter()
        .map(mapper)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}
