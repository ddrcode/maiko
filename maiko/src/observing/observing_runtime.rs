use std::{collections::HashMap, sync::Arc};

use tokio::{select, sync::mpsc::Receiver};
use tokio_util::sync::CancellationToken;

use crate::{
    Event, Topic,
    observing::{Observer, ObserverEvent},
};

pub(crate) struct ObservingRuntime<E: Event, T: Topic<E>> {
    receiver: Receiver<ObserverEvent<E, T>>,
    observers: HashMap<u8, Box<dyn Observer<E, T>>>,
    last_id: u8,
    paused: bool,
    cancel_token: Arc<CancellationToken>,
}

impl<E: Event, T: Topic<E>> ObservingRuntime<E, T> {
    pub fn new(
        receiver: Receiver<ObserverEvent<E, T>>,
        cancel_token: Arc<CancellationToken>,
    ) -> Self {
        Self {
            receiver,
            cancel_token,
            observers: HashMap::new(),
            last_id: 0,
            paused: false,
        }
    }

    pub async fn run(&mut self) {
        loop {
            select! {
                _ = self.cancel_token.cancelled() => {
                    break;
                }
                Some(event) = self.receiver.recv() => {
                    self.handle_event(event).await;
                }
            }
        }
    }

    async fn handle_event(&mut self, event: ObserverEvent<E, T>) {
        use ObserverEvent::*;
        match event {
            AddObserver(obs, resp) => {
                let id = self.last_id;
                self.observers.insert(id, obs);
                self.last_id += 1;
                let _ = resp.send(id);
            }
            RemoveObserver(id) => {
                self.observers.remove(&id);
            }
            Pause => {
                self.paused = true;
            }
            Resume => {
                self.paused = false;
            }
            EventSent(event, topic, actor_id) if !self.paused => {
                self.observers
                    .values()
                    .for_each(|o| o.on_event_sent(&event, &topic, &actor_id));
            }

            _ => {}
        }
    }

    fn run_observers(&self, event: ObserverEvent<E, T>) {}
}
