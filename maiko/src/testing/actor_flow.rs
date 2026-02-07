//! Actor flow view for querying which actors were visited by an event chain.

use std::collections::HashSet;

use crate::{ActorId, Event, Topic};

use super::EventChain;

/// Actor flow view for querying which actors were visited by the chain.
pub struct ActorFlow<'a, E: Event, T: Topic<E>> {
    pub(super) chain: &'a EventChain<E, T>,
}

impl<E: Event, T: Topic<E>> ActorFlow<'_, E, T> {
    /// Returns the list of actors that received events in this chain.
    fn receivers(&self) -> Vec<&ActorId> {
        self.chain
            .chain_entries()
            .map(|e| e.receiver())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Returns the ordered list of actors that received events (in BFS order).
    fn ordered_receivers(&self) -> Vec<&ActorId> {
        let mut seen = HashSet::new();
        self.chain
            .ordered_entries()
            .into_iter()
            .filter_map(|e| {
                let actor = e.receiver();
                if seen.insert(actor) {
                    Some(actor)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns true if all specified actors received events from this chain (any order).
    pub fn visited_all(&self, actors: &[&ActorId]) -> bool {
        let receivers: HashSet<_> = self.receivers().into_iter().collect();
        actors.iter().all(|a| receivers.contains(*a))
    }

    /// Returns true if events passed through the specified actors in order (gaps allowed).
    pub fn through(&self, actors: &[&ActorId]) -> bool {
        if actors.is_empty() {
            return true;
        }

        let ordered = self.ordered_receivers();
        let mut actor_iter = actors.iter();
        let mut current = actor_iter.next();

        for receiver in &ordered {
            if let Some(expected) = current {
                if **receiver == **expected {
                    current = actor_iter.next();
                }
            }
        }

        current.is_none() // All actors were found in order
    }

    /// Returns true if the chain visited exactly these actors in this order.
    pub fn exactly(&self, actors: &[&ActorId]) -> bool {
        let ordered = self.ordered_receivers();
        if ordered.len() != actors.len() {
            return false;
        }
        ordered.iter().zip(actors.iter()).all(|(a, b)| **a == **b)
    }
}
