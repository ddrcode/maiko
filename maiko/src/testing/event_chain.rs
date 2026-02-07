//! Event chain tracing for testing event propagation.
//!
//! An `EventChain` represents the tree of events spawned from a root event,
//! tracked via correlation IDs. Use it to verify that events propagate
//! through the expected actors and trigger the expected child events.

use std::collections::{HashMap, HashSet};

use crate::{ActorId, Event, EventId, Label, Topic};

use super::{ActorFlow, EventEntry, EventFlow, EventMatcher, EventRecords};

/// A chain of events originating from a single root event.
///
/// The chain captures the tree structure of event propagation via correlation IDs.
/// Use `actors()` to query actor flow or `events()` to query event flow.
///
/// # Example
///
/// ```ignore
/// let chain = harness.chain(root_event_id);
///
/// // Verify actor flow
/// assert!(chain.actors().through(&[&scanner, &pipeline, &writer]));
///
/// // Verify event sequence
/// assert!(chain.events().sequence(&["KeyPress", "HidReport"]));
///
/// // Check branching
/// assert!(chain.diverges_after("KeyPress"));
/// ```
pub struct EventChain<E: Event, T: Topic<E>> {
    root_id: EventId,
    records: EventRecords<E, T>,
    /// All event IDs in this chain (root + all descendants)
    chain_ids: HashSet<EventId>,
    /// Parent -> Children mapping
    children_map: HashMap<EventId, Vec<EventId>>,
}

impl<E: Event, T: Topic<E>> EventChain<E, T> {
    /// Create a new event chain starting from the given root event ID.
    pub(crate) fn new(records: EventRecords<E, T>, root_id: EventId) -> Self {
        let mut chain_ids = HashSet::new();
        let mut children_map: HashMap<EventId, Vec<EventId>> = HashMap::new();

        // Build the tree structure from correlation IDs
        // First, collect all unique event IDs and their correlation relationships
        let mut event_correlations: HashMap<EventId, Option<EventId>> = HashMap::new();
        for entry in &records {
            let id = entry.id();
            let correlation = entry.meta().correlation_id();
            event_correlations.entry(id).or_insert(correlation);
        }

        // Find all descendants of root_id using BFS
        let mut queue = vec![root_id];
        chain_ids.insert(root_id);

        while let Some(current_id) = queue.pop() {
            // Find all events that have current_id as their correlation
            for (id, correlation) in &event_correlations {
                if *correlation == Some(current_id) && !chain_ids.contains(id) {
                    chain_ids.insert(*id);
                    queue.push(*id);
                    children_map.entry(current_id).or_default().push(*id);
                }
            }
        }

        Self {
            root_id,
            records,
            chain_ids,
            children_map,
        }
    }

    /// Returns an actor flow view for querying actor-based patterns.
    pub fn actors(&self) -> ActorFlow<'_, E, T> {
        ActorFlow { chain: self }
    }

    /// Returns an event flow view for querying event-based patterns.
    pub fn events(&self) -> EventFlow<'_, E, T> {
        EventFlow { chain: self }
    }

    /// Returns true if the chain diverges (has multiple children) after the specified event.
    ///
    /// This is useful for testing fan-out patterns where one event triggers multiple
    /// independent processing paths.
    pub fn diverges_after(&self, matcher: impl Into<EventMatcher<E, T>>) -> bool
    where
        E: Label,
    {
        let matcher = matcher.into();
        for entry in self.chain_entries() {
            if matcher.matches(entry) {
                let id = entry.id();
                if let Some(children) = self.children_map.get(&id) {
                    return children.len() > 1;
                }
            }
        }
        false
    }

    /// Returns the number of branches after the specified event.
    pub fn branches_after(&self, matcher: impl Into<EventMatcher<E, T>>) -> usize
    where
        E: Label,
    {
        let matcher = matcher.into();
        for entry in self.chain_entries() {
            if matcher.matches(entry) {
                let id = entry.id();
                return self.children_map.get(&id).map(|c| c.len()).unwrap_or(0);
            }
        }
        0
    }

    /// Returns a sub-chain representing the path to a specific actor.
    ///
    /// The path includes all events from the root to any event received by the target actor.
    pub fn path_to(&self, actor: &ActorId) -> EventChain<E, T> {
        // Find events received by this actor in the chain
        let target_ids: HashSet<EventId> = self
            .chain_entries()
            .filter(|e| e.receiver() == actor)
            .map(|e| e.id())
            .collect();

        if target_ids.is_empty() {
            // No path to this actor
            return EventChain {
                root_id: self.root_id,
                records: vec![],
                chain_ids: HashSet::new(),
                children_map: HashMap::new(),
            };
        }

        // Trace back from target to root, collecting all events on the path
        let mut path_ids = HashSet::new();
        let mut to_process: Vec<EventId> = target_ids.into_iter().collect();

        // Build reverse mapping (child -> parent)
        let mut parent_map: HashMap<EventId, EventId> = HashMap::new();
        for (parent, children) in &self.children_map {
            for child in children {
                parent_map.insert(*child, *parent);
            }
        }

        while let Some(id) = to_process.pop() {
            if path_ids.insert(id) {
                if let Some(parent) = parent_map.get(&id) {
                    to_process.push(*parent);
                }
            }
        }

        // Filter records and rebuild children_map for the path
        let path_records: Vec<_> = self
            .records
            .iter()
            .filter(|e| path_ids.contains(&e.id()))
            .cloned()
            .collect();

        let path_children: HashMap<_, _> = self
            .children_map
            .iter()
            .filter(|(k, _)| path_ids.contains(k))
            .map(|(k, v)| {
                let filtered: Vec<_> = v
                    .iter()
                    .filter(|id| path_ids.contains(id))
                    .copied()
                    .collect();
                (*k, filtered)
            })
            .filter(|(_, v)| !v.is_empty())
            .collect();

        EventChain {
            root_id: self.root_id,
            records: path_records,
            chain_ids: path_ids,
            children_map: path_children,
        }
    }

    /// Returns an iterator over all entries in this chain.
    pub(super) fn chain_entries(&self) -> impl Iterator<Item = &EventEntry<E, T>> {
        self.records
            .iter()
            .filter(|e| self.chain_ids.contains(&e.id()))
    }

    /// Returns events in order (BFS from root).
    pub(super) fn ordered_entries(&self) -> Vec<&EventEntry<E, T>> {
        let mut result = Vec::new();
        let mut queue = vec![self.root_id];
        let mut visited = HashSet::new();

        // Build id -> entries map (an event can have multiple entries for different receivers)
        let entries_by_id: HashMap<EventId, Vec<&EventEntry<E, T>>> = self
            .records
            .iter()
            .filter(|e| self.chain_ids.contains(&e.id()))
            .fold(HashMap::new(), |mut acc, entry| {
                acc.entry(entry.id()).or_default().push(entry);
                acc
            });

        while let Some(id) = queue.pop() {
            if visited.insert(id) {
                if let Some(entries) = entries_by_id.get(&id) {
                    result.extend(entries.iter().copied());
                }
                if let Some(children) = self.children_map.get(&id) {
                    queue.extend(children.iter().copied());
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DefaultTopic, Envelope, Label};
    use std::borrow::Cow;
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    enum TestEvent {
        Start,
        Process,
        Complete,
        Branch,
    }

    impl Event for TestEvent {}

    impl Label for TestEvent {
        fn label(&self) -> Cow<'static, str> {
            Cow::Borrowed(match self {
                TestEvent::Start => "Start",
                TestEvent::Process => "Process",
                TestEvent::Complete => "Complete",
                TestEvent::Branch => "Branch",
            })
        }
    }

    fn topic() -> Arc<DefaultTopic> {
        Arc::new(DefaultTopic)
    }

    fn actor(name: &str) -> ActorId {
        ActorId::new(Arc::from(name))
    }

    /// Helper to build a chain of events for testing.
    /// Returns (records, root_id) where records is a Vec of EventEntry.
    fn build_linear_chain() -> (EventRecords<TestEvent, DefaultTopic>, EventId) {
        // Chain: Start -> Process -> Complete
        // Actors: alice -> bob -> charlie
        let alice = actor("alice");
        let bob = actor("bob");
        let charlie = actor("charlie");
        let t = topic();

        // Root event: Start from alice to bob
        let start = Arc::new(Envelope::new(TestEvent::Start, alice.clone()));
        let start_id = start.id();
        let start_entry = EventEntry::new(start, t.clone(), bob.clone());

        // Child: Process from bob (correlated to start) to charlie
        let process = Arc::new(Envelope::with_correlation(
            TestEvent::Process,
            bob.clone(),
            start_id,
        ));
        let process_id = process.id();
        let process_entry = EventEntry::new(process, t.clone(), charlie.clone());

        // Grandchild: Complete from charlie (correlated to process) to alice
        let complete = Arc::new(Envelope::with_correlation(
            TestEvent::Complete,
            charlie,
            process_id,
        ));
        let complete_entry = EventEntry::new(complete, t, alice);

        (vec![start_entry, process_entry, complete_entry], start_id)
    }

    /// Build a branching chain: Start -> [Process, Branch]
    fn build_branching_chain() -> (EventRecords<TestEvent, DefaultTopic>, EventId) {
        let alice = actor("alice");
        let bob = actor("bob");
        let charlie = actor("charlie");
        let t = topic();

        // Root: Start from alice
        let start = Arc::new(Envelope::new(TestEvent::Start, alice.clone()));
        let start_id = start.id();
        let start_entry = EventEntry::new(start, t.clone(), bob.clone());

        // Branch 1: Process from bob (correlated to start) to charlie
        let process = Arc::new(Envelope::with_correlation(
            TestEvent::Process,
            bob.clone(),
            start_id,
        ));
        let process_entry = EventEntry::new(process, t.clone(), charlie.clone());

        // Branch 2: Branch from bob (correlated to start) to alice
        let branch = Arc::new(Envelope::with_correlation(TestEvent::Branch, bob, start_id));
        let branch_entry = EventEntry::new(branch, t, alice);

        (vec![start_entry, process_entry, branch_entry], start_id)
    }

    // ==================== ActorFlow Tests ====================

    #[test]
    fn actor_flow_visited_all_returns_true_when_all_present() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        let bob = actor("bob");
        let charlie = actor("charlie");
        let alice = actor("alice");

        assert!(chain.actors().visited_all(&[&bob, &charlie, &alice]));
    }

    #[test]
    fn actor_flow_visited_all_returns_false_when_missing() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        let bob = actor("bob");
        let dave = actor("dave");

        assert!(!chain.actors().visited_all(&[&bob, &dave]));
    }

    #[test]
    fn actor_flow_through_returns_true_for_correct_order() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        let bob = actor("bob");
        let charlie = actor("charlie");
        let alice = actor("alice");

        // bob -> charlie -> alice (order they received events)
        assert!(chain.actors().through(&[&bob, &charlie, &alice]));
    }

    #[test]
    fn actor_flow_through_allows_gaps() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        let bob = actor("bob");
        let alice = actor("alice");

        // bob -> alice with charlie skipped
        assert!(chain.actors().through(&[&bob, &alice]));
    }

    #[test]
    fn actor_flow_through_returns_false_for_wrong_order() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        let bob = actor("bob");
        let alice = actor("alice");

        // alice appears after charlie, so alice -> bob is wrong order
        assert!(!chain.actors().through(&[&alice, &bob]));
    }

    #[test]
    fn actor_flow_exactly_returns_true_for_exact_match() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        let bob = actor("bob");
        let charlie = actor("charlie");
        let alice = actor("alice");

        assert!(chain.actors().exactly(&[&bob, &charlie, &alice]));
    }

    #[test]
    fn actor_flow_exactly_returns_false_for_partial() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        let bob = actor("bob");
        let charlie = actor("charlie");

        assert!(!chain.actors().exactly(&[&bob, &charlie]));
    }

    #[test]
    fn actor_flow_exactly_returns_false_for_wrong_order() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        let bob = actor("bob");
        let charlie = actor("charlie");
        let alice = actor("alice");

        assert!(!chain.actors().exactly(&[&bob, &alice, &charlie]));
    }

    // ==================== EventFlow Tests ====================

    #[test]
    fn event_flow_contains_finds_event_by_label() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        assert!(chain.events().contains("Start"));
        assert!(chain.events().contains("Process"));
        assert!(chain.events().contains("Complete"));
        assert!(!chain.events().contains("Branch"));
    }

    #[test]
    fn event_flow_through_matches_order_with_gaps() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        assert!(chain.events().through(&["Start", "Complete"]));
        assert!(chain.events().through(&["Start", "Process", "Complete"]));
    }

    #[test]
    fn event_flow_through_returns_false_for_wrong_order() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        assert!(!chain.events().through(&["Complete", "Start"]));
    }

    #[test]
    fn event_flow_sequence_requires_consecutive() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        // Consecutive: Start -> Process -> Complete
        assert!(chain.events().sequence(&["Start", "Process", "Complete"]));
        assert!(chain.events().sequence(&["Start", "Process"]));
        assert!(chain.events().sequence(&["Process", "Complete"]));

        // Empty sequence is always true
        let empty: &[&str] = &[];
        assert!(chain.events().sequence(empty));
    }

    // ==================== Branching Tests ====================

    #[test]
    fn diverges_after_detects_branching() {
        let (records, root_id) = build_branching_chain();
        let chain = EventChain::new(records, root_id);

        // Start has two children (Process and Branch)
        assert!(chain.diverges_after("Start"));

        // Process and Branch have no children
        assert!(!chain.diverges_after("Process"));
        assert!(!chain.diverges_after("Branch"));
    }

    #[test]
    fn diverges_after_returns_false_for_linear() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        assert!(!chain.diverges_after("Start"));
        assert!(!chain.diverges_after("Process"));
    }

    #[test]
    fn branches_after_counts_children() {
        let (records, root_id) = build_branching_chain();
        let chain = EventChain::new(records, root_id);

        assert_eq!(chain.branches_after("Start"), 2);
        assert_eq!(chain.branches_after("Process"), 0);
        assert_eq!(chain.branches_after("NonExistent"), 0);
    }

    // ==================== Path Tests ====================

    #[test]
    fn path_to_extracts_subchain() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        let charlie = actor("charlie");
        let path = chain.path_to(&charlie);

        // Path should include Start -> Process (which charlie receives)
        assert!(path.events().contains("Start"));
        assert!(path.events().contains("Process"));
        // Complete is sent by charlie, not received by charlie, so it shouldn't be in the path
    }

    #[test]
    fn path_to_returns_empty_for_unknown_actor() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        let unknown = actor("unknown");
        let path = chain.path_to(&unknown);

        assert!(!path.events().contains("Start"));
        assert!(!path.events().contains("Process"));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn empty_chain_handles_gracefully() {
        let chain: EventChain<TestEvent, DefaultTopic> = EventChain::new(vec![], 0);

        assert!(!chain.diverges_after("Anything"));
        assert_eq!(chain.branches_after("Anything"), 0);
        assert!(chain.actors().visited_all(&[]));
        assert!(chain.events().through(&[] as &[&str]));
    }

    #[test]
    fn through_with_empty_matchers_returns_true() {
        let (records, root_id) = build_linear_chain();
        let chain = EventChain::new(records, root_id);

        let empty_actors: &[&ActorId] = &[];
        let empty_events: &[&str] = &[];

        assert!(chain.actors().through(empty_actors));
        assert!(chain.events().through(empty_events));
    }
}
