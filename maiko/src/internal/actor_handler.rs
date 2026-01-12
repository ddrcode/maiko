use std::{pin::Pin, sync::Arc};

use tokio::{select, sync::mpsc::Receiver, time::Sleep};
use tokio_util::sync::CancellationToken;

use crate::{Actor, Context, Envelope, Result, StepAction};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum StepPause {
    #[default]
    None,
    AwaitEvent,
    Suppressed,
}

#[derive(Default)]
struct StepHandler {
    backoff: Option<Pin<Box<Sleep>>>,
    pause: StepPause,
}

impl StepHandler {
    fn is_delayed(&self) -> bool {
        self.backoff.is_some() && self.pause == StepPause::None
    }
    fn can_step(&self) -> bool {
        self.backoff.is_none() && self.pause == StepPause::None
    }
}

pub(crate) struct ActorHandler<A: Actor> {
    pub(crate) actor: A,
    pub(crate) receiver: Receiver<Arc<Envelope<A::Event>>>,
    pub(crate) ctx: Context<A::Event>,
    pub(crate) max_events_per_tick: usize,
    pub(crate) cancel_token: Arc<CancellationToken>,
}

impl<A: Actor> ActorHandler<A> {
    pub async fn run(&mut self) -> Result<()> {
        self.actor.on_start().await?;
        let token = self.cancel_token.clone();
        let mut step_handler = StepHandler::default();
        while self.ctx.is_alive() {
            select! {
                biased;

                _ = token.cancelled() => {
                    self.ctx.stop();
                    break;
                },

                Some(event) = self.receiver.recv() => {
                    let res = self.actor.handle_envelope(&event).await;
                    self.handle_error(res)?;

                    let mut cnt = 1;
                    while let Ok(event) = self.receiver.try_recv() {
                        let res = self.actor.handle_envelope(&event).await;
                        self.handle_error(res)?;
                        cnt += 1;
                        if cnt == self.max_events_per_tick {
                            break;
                        }
                    }
                    if step_handler.pause == StepPause::AwaitEvent {
                        step_handler.pause = StepPause::None;
                    }
                }

                _ = async {
                    if let Some(backoff_sleep) = step_handler.backoff.as_mut() {
                        backoff_sleep.as_mut().await;
                    }
                }, if step_handler.is_delayed() => {
                    let _ = step_handler.backoff.take();
                    match self.actor.step().await {
                        Ok(action) => handle_step_action(action, &mut step_handler).await,
                        Err(e) => self.actor.on_error(e)?
                     }
                }


                res = self.actor.step(), if step_handler.can_step() => {
                     match res {
                        Ok(action) => handle_step_action(action, &mut step_handler).await,
                        Err(e) => self.actor.on_error(e)?
                     }
                }
            }
        }

        self.actor.on_shutdown().await
    }

    #[inline]
    fn handle_error<T>(&self, result: Result<T>) -> Result<()> {
        if let Err(e) = result {
            self.actor.on_error(e)?;
        }
        Ok(())
    }
}

async fn handle_step_action(step_action: StepAction, step_handler: &mut StepHandler) {
    let pause = match step_action {
        crate::StepAction::Continue => StepPause::None,
        crate::StepAction::Yield => {
            tokio::task::yield_now().await;
            StepPause::None
        }
        crate::StepAction::AwaitEvent => StepPause::AwaitEvent,
        crate::StepAction::Backoff(duration) => {
            step_handler
                .backoff
                .replace(Box::pin(tokio::time::sleep(duration)));
            StepPause::None
        }
        crate::StepAction::Never => StepPause::Suppressed,
    };
    step_handler.pause = pause;
}
