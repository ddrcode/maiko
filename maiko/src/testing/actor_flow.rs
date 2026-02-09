//! Actor flow view for querying which actors were visited by an event chain.

use std::collections::HashSet;

use crate::{ActorId, Event, Topic};

use super::EventChain;

/// Actor flow view for querying which actors were visited by the chain.
pub struct ActorFlow<'a, E: Event, T: Topic<E>> {
    pub(super) chain: &'a EventChain<E, T>,
}

impl<E: Event, T: Topic<E>> ActorFlow<'_, E, T> {
    /// Returns all actors involved in this chain (sender + receivers).
    ///
    /// The list includes the root event's sender followed by all actors
    /// that received events. Order is not guaranteed.
    pub fn all(&self) -> Vec<&ActorId> {
        let mut actors: HashSet<&ActorId> =
            self.chain.chain_entries().map(|e| e.receiver()).collect();

        // Include root sender
        if let Some(sender) = self.chain.root_sender() {
            actors.insert(sender);
        }

        actors.into_iter().collect()
    }

    /// Returns actors in order of participation (BFS order).
    ///
    /// Starts with the root event's sender, followed by receivers in the
    /// order they appear in the chain traversal.
    pub fn ordered(&self) -> Vec<&ActorId> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        // Start with root sender
        if let Some(sender) = self.chain.root_sender() {
            if seen.insert(sender) {
                result.push(sender);
            }
        }

        // Add receivers in BFS order
        for entry in self.chain.ordered_entries() {
            let actor = entry.receiver();
            if seen.insert(actor) {
                result.push(actor);
            }
        }

        result
    }

    /// Returns true if all specified actors participated in this chain (any order).
    pub fn visited_all(&self, actors: &[&ActorId]) -> bool {
        let all_actors: HashSet<_> = self.all().into_iter().collect();
        actors.iter().all(|a| all_actors.contains(*a))
    }

    /// Returns true if actors participated in the specified order (gaps allowed).
    pub fn through(&self, actors: &[&ActorId]) -> bool {
        if actors.is_empty() {
            return true;
        }

        let ordered = self.ordered();
        let mut actor_iter = actors.iter();
        let mut current = actor_iter.next();

        for participant in &ordered {
            if let Some(expected) = current {
                if **participant == **expected {
                    current = actor_iter.next();
                }
            }
        }

        current.is_none() // All actors were found in order
    }

    /// Returns true if the chain involved exactly these actors in this order.
    pub fn exactly(&self, actors: &[&ActorId]) -> bool {
        let ordered = self.ordered();
        if ordered.len() != actors.len() {
            return false;
        }
        ordered.iter().zip(actors.iter()).all(|(a, b)| **a == **b)
    }
}
