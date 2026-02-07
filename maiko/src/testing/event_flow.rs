//! Event flow view for querying the sequence of events in a chain.

use std::collections::HashSet;

use crate::{Event, Label, Topic};

use super::{EventChain, EventEntry, EventMatcher};

/// Event flow view for querying the sequence of events in the chain.
pub struct EventFlow<'a, E: Event, T: Topic<E>> {
    pub(super) chain: &'a EventChain<E, T>,
}

impl<E: Event + Label, T: Topic<E>> EventFlow<'_, E, T> {
    /// Returns true if the chain contains an event matching the given matcher.
    pub fn contains(&self, matcher: impl Into<EventMatcher<E, T>>) -> bool {
        let matcher = matcher.into();
        self.chain.chain_entries().any(|e| matcher.matches(e))
    }

    /// Returns true if events matching the matchers appear consecutively in the chain.
    pub fn sequence<M>(&self, matchers: &[M]) -> bool
    where
        M: Into<EventMatcher<E, T>> + Clone,
    {
        if matchers.is_empty() {
            return true;
        }

        let ordered = self.ordered_events();
        let matchers: Vec<_> = matchers.iter().cloned().map(|m| m.into()).collect();

        // Look for consecutive matches
        'outer: for start in 0..ordered.len() {
            if matchers[0].matches(ordered[start]) {
                let mut match_idx = 1;
                for entry in ordered.iter().skip(start + 1) {
                    if match_idx >= matchers.len() {
                        return true;
                    }
                    if matchers[match_idx].matches(entry) {
                        match_idx += 1;
                    } else {
                        continue 'outer;
                    }
                }
                if match_idx == matchers.len() {
                    return true;
                }
            }
        }
        false
    }

    /// Returns true if events matching the matchers appear in order (gaps allowed).
    pub fn through<M>(&self, matchers: &[M]) -> bool
    where
        M: Into<EventMatcher<E, T>> + Clone,
    {
        if matchers.is_empty() {
            return true;
        }

        let ordered = self.ordered_events();
        let matchers: Vec<_> = matchers.iter().cloned().map(|m| m.into()).collect();
        let mut matcher_idx = 0;

        for entry in &ordered {
            if matcher_idx >= matchers.len() {
                break;
            }
            if matchers[matcher_idx].matches(entry) {
                matcher_idx += 1;
            }
        }

        matcher_idx == matchers.len()
    }

    /// Returns ordered unique events (by label, BFS order).
    fn ordered_events(&self) -> Vec<&EventEntry<E, T>> {
        let mut seen_ids = HashSet::new();
        self.chain
            .ordered_entries()
            .into_iter()
            .filter(|e| seen_ids.insert(e.id()))
            .collect()
    }
}
