use crate::{ActorId, Envelope, Error, Event, StepAction, Topic};

pub trait Observer<E: Event, T: Topic<E>> {
    fn on_event_sent(&self, envelope: &Envelope<E>, topic: &T, receiver: &ActorId) {
        let _e = envelope;
        let _t = topic;
        let _r = receiver;
    }

    fn on_event_received(&self, envelope: &Envelope<E>, receiver: &ActorId) {
        let _e = envelope;
        let _r = receiver;
    }

    fn on_event_processed(&self, envelope: &Envelope<E>, actor_id: &ActorId) {
        let _e = envelope;
        let _a = actor_id;
    }

    fn on_error(&self, err: Error, actor_id: &ActorId) {
        let _a = actor_id;
        let _e = err;
    }

    fn on_step_call(&self, actor_id: &ActorId) {
        let _a = actor_id;
    }

    fn on_step_exit(&self, step_action: &StepAction, actor_id: &ActorId) {
        let _s = step_action;
        let _a = actor_id;
    }
}
