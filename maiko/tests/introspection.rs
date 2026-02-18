#![cfg(feature = "introspection")]

use maiko::{Actor, ActorId, Envelope, Event, Result, Subscribe, Supervisor, DefaultTopic};
use std::sync::Arc;
use tokio::sync::Barrier;

#[derive(Event, Debug, Clone)]
enum TestEvent {
    #[allow(dead_code)]
    Sensor(f64),
}


struct DummyActor;
impl Actor for DummyActor {
    type Event = TestEvent;
    async fn handle_event(&mut self, _: &Envelope<Self::Event>) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_introspection_list_actors() {
    let mut sup = Supervisor::<TestEvent>::default();
    sup.add_actor("a", |_| DummyActor, Subscribe::none()).unwrap();
    sup.add_actor("b", |_| DummyActor, Subscribe::none()).unwrap();

    sup.start().await.unwrap();

    // Give a moment for registration to settle if needed, though they are registered synchronously
    
    let actors = sup.introspection().list_actors();
    assert_eq!(actors.len(), 2);
    
    let names: Vec<_> = actors.iter().map(|a| a.actor_id.name()).collect();
    assert!(names.contains(&"a"));
    assert!(names.contains(&"b"));
}

#[tokio::test]
async fn test_introspection_actor_info() {
    let mut sup = Supervisor::<TestEvent>::default();
    let id = sup.add_actor("my_actor", |_| DummyActor, Subscribe::none()).unwrap();

    sup.start().await.unwrap();

    let info = sup.introspection().actor_info(&id).expect("Actor not found");
    assert_eq!(info.actor_id.name(), "my_actor");
    assert_eq!(info.mailbox_depth, 0);
}

#[tokio::test]
async fn test_introspection_snapshot() {
    let mut sup = Supervisor::<TestEvent>::default();
    sup.add_actor("a", |_| DummyActor, Subscribe::none()).unwrap();

    let snap = sup.introspection().snapshot();
    assert!(snap.timestamp > 0);
    assert_eq!(snap.actors.len(), 1);
    assert_eq!(snap.actors[0].actor_id.name(), "a");
}

#[tokio::test]
async fn test_introspection_queue_depth() {
    struct BlockingActor {
        barrier: Arc<Barrier>,
    }

    impl Actor for BlockingActor {
        type Event = TestEvent;
        async fn handle_event(&mut self, _: &Envelope<Self::Event>) -> Result<()> {
            self.barrier.wait().await;
            Ok(())
        }
    }

    let mut sup = Supervisor::<TestEvent>::default();
    let barrier = Arc::new(Barrier::new(2)); // Wait for test + actor
    let b = barrier.clone();
    
    // Register actor
    // The topic needs to be explicitly provided. 
    // DefaultTopic works if we use DefaultTopic in Supervisor, but we need to subscribe to something.
    // If we use DefaultTopic, we can just use `&[DefaultTopic]`.
    sup.add_actor("blocker", move |_| BlockingActor { barrier: b }, &[DefaultTopic]).unwrap();

    sup.start().await.unwrap();

    // Send an event to trigger the barrier wait in the actor
    sup.send(TestEvent::Sensor(1.0)).await.unwrap();

    // Send more events to fill the queue
    sup.send(TestEvent::Sensor(2.0)).await.unwrap();
    sup.send(TestEvent::Sensor(3.0)).await.unwrap();

    // Give a moment for the broker to dispatch
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let info = sup.introspection().actor_info(&ActorId::new("blocker".into())).unwrap();
    
    assert!(info.mailbox_depth > 0, "Queue depth should be positive, got {}", info.mailbox_depth);
    assert_eq!(info.status, maiko::introspection::ActorStatus::Processing);

    // Unblock
    barrier.wait().await;
}
