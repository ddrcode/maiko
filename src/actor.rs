use crate::{Context, Event, Result};

pub trait Actor {
    type Event: Event;

    async fn handle(&mut self, event: Self::Event) -> Result<Option<Self::Event>>;
    fn ctx(&self) -> &Context<Self::Event>;
    fn ctx_mut(&mut self) -> &mut Context<Self::Event>;
    fn set_ctx(&mut self, ctx: Context<Self::Event>) -> Result<()>;
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
