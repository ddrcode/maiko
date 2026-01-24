use crate::{
    ActorHandle, Event, EventId, Topic,
    testing::{EventEntry, EventRecords},
};

pub type Filters<E, T> = Vec<Box<dyn Fn(&EventEntry<E, T>) -> bool>>;

pub struct EventQuery<E: Event, T: Topic<E>> {
    events: EventRecords<E, T>,
    filters: Filters<E, T>,
}

impl<E: Event, T: Topic<E>> EventQuery<E, T> {
    pub(crate) fn new(events: EventRecords<E, T>) -> Self {
        Self {
            events,
            filters: Vec::new(),
        }
    }

    pub(crate) fn with_filter<F>(events: EventRecords<E, T>, filter: F) -> Self
    where
        F: 'static + Fn(&EventEntry<E, T>) -> bool,
    {
        let filters: Filters<E, T> = vec![Box::new(filter)];
        Self { events, filters }
    }

    fn add_filter<F>(&mut self, filter: F)
    where
        F: 'static + Fn(&EventEntry<E, T>) -> bool,
    {
        self.filters.push(Box::new(filter));
    }

    fn apply_filters(&self) -> Vec<&EventEntry<E, T>> {
        self.events
            .iter()
            .filter(|e| {
                for filter in &self.filters {
                    if !filter(e) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    pub(crate) fn records(&self) -> EventRecords<E, T> {
        self.events.clone()
    }

    pub fn count(&self) -> usize {
        self.apply_filters().len()
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    pub fn first(&self) -> Option<EventEntry<E, T>> {
        self.apply_filters().into_iter().next().cloned()
    }

    pub fn last(&self) -> Option<EventEntry<E, T>> {
        self.apply_filters().into_iter().last().cloned()
    }

    pub fn into_iter(&self) -> impl Iterator<Item = EventEntry<E, T>> + '_ {
        self.apply_filters().into_iter().cloned()
    }

    pub fn iter(&self) -> impl Iterator<Item = &EventEntry<E, T>> + '_ {
        self.apply_filters().into_iter()
    }

    pub fn collect(&self) -> Vec<EventEntry<E, T>> {
        self.apply_filters().into_iter().cloned().collect()
    }

    pub fn all(&self, predicate: impl Fn(&EventEntry<E, T>) -> bool) -> bool {
        self.apply_filters().into_iter().all(predicate)
    }

    pub fn any(&self, predicate: impl Fn(&EventEntry<E, T>) -> bool) -> bool {
        self.apply_filters().into_iter().any(predicate)
    }

    pub fn sent_by(self, actor: ActorHandle) -> Self {
        let mut res = self;
        res.add_filter(move |e| e.sender_actor_eq(actor.name()));
        res
    }

    pub fn received_by(self, actor: ActorHandle) -> Self {
        let mut res = self;
        res.add_filter(move |e| e.actor_eq(actor.name()));
        res
    }

    pub fn with_topic(self, topic: T) -> Self {
        let mut res = self;
        res.add_filter(move |e| e.topic == topic);
        res
    }

    pub fn with_event(self, id: impl Into<EventId>) -> Self {
        let id = id.into();
        let mut res = self;
        res.add_filter(move |e| e.event.id() == id);
        res
    }

    pub fn matching<F>(self, predicate: F) -> Self
    where
        F: 'static + Fn(&EventEntry<E, T>) -> bool,
    {
        let mut res = self;
        res.add_filter(predicate);
        res
    }

    pub fn after(self, event: &EventEntry<E, T>) -> Self {
        let timestamp = event.event.meta().timestamp();
        let mut res = self;
        res.add_filter(move |e| e.event.meta().timestamp() > timestamp);
        res
    }

    pub fn before(self, event: &EventEntry<E, T>) -> Self {
        let timestamp = event.event.meta().timestamp();
        let mut res = self;
        res.add_filter(move |e| e.event.meta().timestamp() < timestamp);
        res
    }

    pub fn correlated_with(self, id: impl Into<EventId>) -> Self {
        let correlation_id = id.into();
        let mut res = self;
        res.add_filter(move |e| {
            if let Some(cid) = e.event.meta().correlation_id() {
                cid == correlation_id
            } else {
                false
            }
        });
        res
    }
}
