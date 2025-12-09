use core::marker::Send;

use async_trait::async_trait;

use crate::{Context, Event, Meta, Result};

#[async_trait]
pub trait Actor: Send {
    type Event: Event + Send;

    #[allow(unused_variables)]
    async fn handle(&mut self, event: &Self::Event, meta: &Meta) -> Result<Option<Self::Event>> {
        Ok(None)
    }

    #[allow(unused_variables)]
    async fn tick(&mut self, ctx: &Context<Self::Event>) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    async fn on_start(&mut self, ctx: &Context<Self::Event>) -> Result<()> {
        Ok(())
    }

    async fn on_shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_init(&mut self, ctx: Context<Self::Event>) {}
}
