pub trait Actor {
    type Event;

    async fn handle(&mut self, event: Self::Event);
}

#[cfg(test)]
mod tests {
    use crate::{Context, Event};

    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_simple_actor() {
        struct MyEvent {}
        impl Event for MyEvent {}

        struct MyActor {
            ctx: Context<MyEvent>,
        }

        impl Actor for MyActor {
            type Event = MyEvent;

            async fn handle(&mut self, event: Self::Event) {
                self.ctx.sender.send(event).await.unwrap();
            }
        }
    }
}
