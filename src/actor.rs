use core::marker::Send;

use async_trait::async_trait;
use tokio::select;

use crate::{Context, Event, Meta, Result};

#[async_trait]
pub trait Actor: Send {
    type Event: Event + Send;

    fn ctx(&self) -> &Context<Self::Event>;
    fn ctx_mut(&mut self) -> &mut Context<Self::Event>;
    fn set_ctx(&mut self, ctx: Context<Self::Event>) -> Result<()>;
    fn name(&self) -> &str;

    async fn handle(&mut self, _event: &Self::Event, _meta: &Meta) -> Result<Option<Self::Event>> {
        Ok(None)
    }

    async fn tick(&mut self) -> Result<()> {
        Ok(())
    }

    async fn send(&mut self, event: Self::Event) -> Result<()> {
        self.ctx().send(event, self.name()).await?;
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        self.start().await?;
        let mut rx = self
            .ctx_mut()
            .receiver
            .take()
            .expect("Receiver already taken");
        let token = self.ctx().cancel_token.clone();
        loop {
            select! {
                _ = token.cancelled() => break,
                Some(event) = rx.recv() => {
                    if let Some(out) = self.handle(&event.event, &event.meta).await? {
            self.send(out).await?;
                    }
                },
                r = self.tick() => r?
            }
        }
        self.shutdown().await
    }

    async fn start(&mut self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    fn exit(&mut self) {
        self.ctx().cancel_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use crate::{Context, Event};

    use super::*;

    #[tokio::test]
    async fn test_simple_actor() {
        #[derive(Clone)]
        struct MyEvent {}
        impl Event for MyEvent {}

        struct MyActor {
            ctx: Context<MyEvent>,
        }

        #[async_trait]
        impl Actor for MyActor {
            type Event = MyEvent;

            async fn handle(
                &mut self,
                event: &Self::Event,
                _meta: &Meta,
            ) -> Result<Option<Self::Event>> {
                self.send(event.clone()).await.unwrap();
                Ok(None)
            }

            fn ctx(&self) -> &Context<Self::Event> {
                &self.ctx
            }
            fn ctx_mut(&mut self) -> &mut Context<Self::Event> {
                &mut self.ctx
            }
            fn set_ctx(&mut self, ctx: Context<Self::Event>) -> Result<()> {
                self.ctx = ctx;
                Ok(())
            }
            fn name(&self) -> &str {
                "MyActor"
            }
        }
    }
}
