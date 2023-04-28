pub mod actor_cell;
pub mod actor_cell_with_ref;
pub mod actor_context;
pub mod actor_path;
pub mod actor_ref;
pub mod actor_ref_provider;
pub mod actor_system;
pub mod address;
pub mod child_state;
pub mod children_refs;
pub mod props;
pub mod scheduler;

use crate::core::actor::actor_context::ActorContext;
use crate::core::actor::actor_ref::ActorRef;
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::message::Message;

use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;
use thiserror::Error;

pub type ActorResult<A> = Result<A, ActorError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ActorError {
  #[error("Actor failed: {message}")]
  ActorFailed { message: String },
}

pub trait ActorBehavior<Msg: Message>: Debug {
  fn around_receive(&mut self, ctx: ActorContext<Msg>, msg: Msg) -> ActorResult<()> {
    self.receive(ctx, msg)
  }

  fn receive(&mut self, ctx: ActorContext<Msg>, msg: Msg) -> ActorResult<()>;

  fn around_pre_restart(&mut self, ctx: ActorContext<Msg>, reason: ActorError, msg: Option<Msg>) -> ActorResult<()> {
    self.pre_restart(ctx, reason, msg)
  }

  fn pre_restart(&mut self, _ctx: ActorContext<Msg>, _reason: ActorError, _msg: Option<Msg>) -> ActorResult<()> {
    log::info!("default pre_start");
    Ok(())
  }

  fn around_pre_start(&mut self, ctx: ActorContext<Msg>) -> ActorResult<()> {
    self.pre_start(ctx)
  }

  fn pre_start(&mut self, _ctx: ActorContext<Msg>) -> ActorResult<()> {
    log::info!("default pre_start");
    Ok(())
  }

  fn around_pre_suspend(&mut self, ctx: ActorContext<Msg>) -> ActorResult<()> {
    self.pre_suspend(ctx)
  }

  fn pre_suspend(&mut self, _ctx: ActorContext<Msg>) -> ActorResult<()> {
    log::info!("default pre_suspend");
    Ok(())
  }

  fn around_post_resume(&mut self, ctx: ActorContext<Msg>, caused_by_failure: Option<ActorError>) -> ActorResult<()> {
    self.post_resume(ctx, caused_by_failure)
  }

  fn post_resume(&mut self, _ctx: ActorContext<Msg>, _caused_by_failure: Option<ActorError>) -> ActorResult<()> {
    log::info!("default post_resume");
    Ok(())
  }

  fn around_post_stop(&mut self, ctx: ActorContext<Msg>) -> ActorResult<()> {
    self.post_stop(ctx)
  }

  fn post_stop(&mut self, _ctx: ActorContext<Msg>) -> ActorResult<()> {
    log::info!("default post_stop");
    Ok(())
  }

  fn around_child_terminated(&mut self, _ctx: ActorContext<Msg>, child: ActorRef<AnyMessage>) -> ActorResult<()> {
    self.child_terminated(child)
  }

  fn child_terminated(&mut self, /* _ctx: ActorContext<Msg>, */ _child: ActorRef<AnyMessage>) -> ActorResult<()> {
    log::info!("default child_terminated");
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct MockActorMutable<Msg: Message> {
  p: PhantomData<Msg>,
}

impl<Msg: Message> ActorBehavior<Msg> for MockActorMutable<Msg> {
  fn receive(&mut self, _ctx: ActorContext<Msg>, _msg: Msg) -> ActorResult<()> {
    Ok(())
  }

  fn child_terminated(&mut self, /* _ctx: ActorContext<Msg>, */ _child: ActorRef<AnyMessage>) -> ActorResult<()> {
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct AnyMessageActorWrapper<Msg: Message, Actor: ActorBehavior<Msg>> {
  inner_actor: Actor,
}

impl<Msg: Message, Actor: ActorBehavior<Msg>> AnyMessageActorWrapper<Msg, Actor> {
  pub fn new(actor: Actor) -> Self {
    Self { inner_actor: actor }
  }
}

impl<Msg: Message, Actor: ActorBehavior<Msg>> ActorBehavior<AnyMessage> for AnyMessageActorWrapper<Msg, Actor> {
  fn receive(&mut self, ctx: ActorContext<AnyMessage>, msg: AnyMessage) -> ActorResult<()> {
    let typed_msg = msg.take::<Msg>().unwrap();
    let typed_ctx = ctx.to_typed(true);
    self.inner_actor.around_receive(typed_ctx, typed_msg)
  }

  fn child_terminated(&mut self, /* _ctx: ActorContext<Msg>, */ child: ActorRef<AnyMessage>) -> ActorResult<()> {
    // let typed_ctx = _ctx.to_typed(false);
    let mut actor = self.inner_actor.borrow_mut();
    actor.child_terminated(child)
  }
}
