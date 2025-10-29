use core::marker::Send;

use tokio::select;

use crate::{Context, Envelope, Event, Result};

pub trait Actor: Send {
    type Event: Event;

    fn ctx(&self) -> &Context<Self::Event>;
    fn ctx_mut(&mut self) -> &mut Context<Self::Event>;
    fn set_ctx(&mut self, ctx: Context<Self::Event>) -> Result<()>;
    fn name(&self) -> &str;

    async fn handle(&mut self, event: &Self::Event) -> Result<Option<Self::Event>>;

    async fn tick(&mut self) -> Result<()> {
        if let Some(event) = self.ctx_mut().receiver.recv().await {
            if let Some(out) = self.handle(&event.event).await? {
                self.send(out).await?;
            }
        }
        Ok(())
    }

    async fn send(&self, event: Self::Event) -> Result<()> {
        if let Err(_) = self.ctx().sender.send(Envelope::new(event)).await {
            return Err(crate::Error::SendError);
        }
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        let is_alive = true;
        while is_alive {
            select! {
                _ = self.tick() => {}
            }
        }
        Ok(())
    }
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

            async fn handle(&mut self, event: Self::Event) -> Result<Option<Self::Event>> {
                self.ctx.sender.send(event).await.unwrap();
                Ok(None)
            }
        }
    }
}
