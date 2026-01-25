use std::rc::Rc;

use crate::{
    ActorHandle, Event, EventId, Topic,
    testing::{EventEntry, EventRecords},
};

type Filter<E, T> = Rc<dyn Fn(&EventEntry<E, T>) -> bool>;

/// A composable query builder for filtering and inspecting recorded events.
///
/// `EventQuery` provides a fluent API for filtering events by various criteria
/// (sender, receiver, topic, correlation, timing) and terminal operations for
/// inspection (count, iteration, assertions).
///
/// # Example
///
/// ```ignore
/// let orders = test.events()
///     .sent_by(&trader)
///     .with_topic(MarketTopic::Order)
///     .after(&initial_tick)
///     .count();
/// ```
#[derive(Clone)]
pub struct EventQuery<E: Event, T: Topic<E>> {
    events: EventRecords<E, T>,
    filters: Vec<Filter<E, T>>,
}

impl<E: Event, T: Topic<E>> EventQuery<E, T> {
    pub(crate) fn new(events: EventRecords<E, T>) -> Self {
        Self {
            events,
            filters: Vec::new(),
        }
    }

    fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&EventEntry<E, T>) -> bool + 'static,
    {
        self.filters.push(Rc::new(filter));
    }

    fn apply_filters(&self) -> Vec<&EventEntry<E, T>> {
        self.events
            .iter()
            .filter(|e| self.filters.iter().all(|f| f(e)))
            .collect()
    }

    // ==================== Terminal Operations ====================

    /// Returns the number of events matching all filters.
    pub fn count(&self) -> usize {
        self.apply_filters().len()
    }

    /// Returns true if no events match the filters.
    pub fn is_empty(&self) -> bool {
        self.apply_filters().is_empty()
    }

    /// Returns the first matching event, if any.
    pub fn first(&self) -> Option<EventEntry<E, T>> {
        self.apply_filters().first().cloned().cloned()
    }

    /// Returns the last matching event, if any.
    pub fn last(&self) -> Option<EventEntry<E, T>> {
        self.apply_filters().last().cloned().cloned()
    }

    /// Returns the nth matching event (0-indexed), if any.
    pub fn nth(&self, index: usize) -> Option<EventEntry<E, T>> {
        self.apply_filters().get(index).cloned().cloned()
    }

    /// Collects all matching events into a Vec.
    pub fn collect(&self) -> Vec<EventEntry<E, T>> {
        self.apply_filters().into_iter().cloned().collect()
    }

    /// Returns true if all matching events satisfy the predicate.
    pub fn all(&self, predicate: impl Fn(&EventEntry<E, T>) -> bool) -> bool {
        self.apply_filters().into_iter().all(predicate)
    }

    /// Returns true if any matching event satisfies the predicate.
    pub fn any(&self, predicate: impl Fn(&EventEntry<E, T>) -> bool) -> bool {
        self.apply_filters().into_iter().any(predicate)
    }

    /// Returns an iterator over references to matching events.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &EventEntry<E, T>> {
        self.apply_filters().into_iter()
    }

    // ==================== Filter Operations ====================

    /// Filter to events sent by the specified actor.
    pub fn sent_by(mut self, actor: &ActorHandle) -> Self {
        let actor = actor.clone();
        self.add_filter(move |e| e.sender_actor_eq(&actor));
        self
    }

    /// Filter to events received by the specified actor.
    pub fn received_by(mut self, actor: &ActorHandle) -> Self {
        let actor = actor.clone();
        self.add_filter(move |e| e.receiver_actor_eq(&actor));
        self
    }

    /// Filter to events with the specified topic.
    pub fn with_topic(mut self, topic: T) -> Self {
        self.add_filter(move |e| e.topic == topic);
        self
    }

    /// Filter to a specific event by ID.
    pub fn with_id(mut self, id: impl Into<EventId>) -> Self {
        let id = id.into();
        self.add_filter(move |e| e.event.id() == id);
        self
    }

    /// Filter using a custom predicate on the event entry.
    pub fn matching<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&EventEntry<E, T>) -> bool + 'static,
    {
        self.add_filter(predicate);
        self
    }

    /// Filter using a custom predicate on the event payload.
    ///
    /// This is a convenience method for filtering by event content without
    /// needing to navigate through the EventEntry structure.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let orders = test.events()
    ///     .matching_event(|e| matches!(e, MarketEvent::Order(_)))
    ///     .count();
    /// ```
    pub fn matching_event<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&E) -> bool + 'static,
    {
        self.add_filter(move |entry| predicate(entry.payload()));
        self
    }

    /// Filter to events occurring after the given event (by timestamp).
    pub fn after(mut self, event: &EventEntry<E, T>) -> Self {
        let timestamp = event.meta().timestamp();
        self.add_filter(move |e| e.meta().timestamp() > timestamp);
        self
    }

    /// Filter to events occurring before the given event (by timestamp).
    pub fn before(mut self, event: &EventEntry<E, T>) -> Self {
        let timestamp = event.meta().timestamp();
        self.add_filter(move |e| e.meta().timestamp() < timestamp);
        self
    }

    /// Filter to events correlated with the given event ID (children).
    pub fn correlated_with(mut self, id: impl Into<EventId>) -> Self {
        let parent_id = id.into();
        self.add_filter(move |e| {
            e.meta().correlation_id() == Some(parent_id)
        });
        self
    }
}
