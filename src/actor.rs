use core::marker::Send;

use async_trait::async_trait;

use crate::{Event, Meta, Result};

#[allow(unused_variables)]
#[async_trait]
pub trait Actor: Send {
    type Event: Event + Send;

    async fn handle(&mut self, event: &Self::Event, meta: &Meta) -> Result<()> {
        Ok(())
    }

    async fn tick(&mut self) -> Result<()> {
        Ok(())
    }

    async fn on_start(&mut self) -> Result<()> {
        Ok(())
    }

    async fn on_shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
