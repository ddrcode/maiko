use std::collections::HashSet;

use crate::{Event, Topic, testing::EventEntry};

pub fn receivers_count<E: Event, T: Topic<E>>(data: &[EventEntry<E, T>]) -> usize {
    data.len()
}

pub fn receivers<E: Event, T: Topic<E>>(data: &[EventEntry<E, T>]) -> Vec<&str> {
    // distinct(data, |e| e.actor_name.as_ref())
    todo!()
}

#[inline]
pub fn distinct<'a, E, T, R>(
    iter: impl Iterator<Item = &'a EventEntry<E, T>> + 'a,
    mapper: impl Fn(&'a EventEntry<E, T>) -> R,
) -> Vec<R>
where
    E: Event,
    T: Topic<E>,
    R: 'a + std::hash::Hash + std::cmp::Eq,
{
    iter.map(mapper)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

#[inline]
pub fn filter_clone<'a, E, T>(
    data: &'a [EventEntry<E, T>],
    filter: impl Fn(&&'a EventEntry<E, T>) -> bool,
) -> Vec<EventEntry<E, T>>
where
    E: Event + Clone,
    T: Topic<E>,
{
    data.iter().filter(filter).cloned().collect()
}
