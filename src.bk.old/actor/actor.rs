use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use anyhow::Result;
use thiserror::Error;

use crate::actor::actor_context::ActorContextRef;
use crate::dispatch::message::Message;

pub type ActorResult<A> = Result<A, ActorError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ActorError {
  #[error("Actor failed: {message}")]
  ActorFailed { message: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReceiveTimeout;

pub trait ActorMutableBehavior: Debug {
  type Msg: Message;
  fn receive(&mut self, ctx: ActorContextRef<Self::Msg>, msg: Self::Msg) -> ActorResult<()>;
  fn around_pre_restart(
    &mut self,
    ctx: ActorContextRef<Self::Msg>,
    reason: ActorError,
    msg: Option<Self::Msg>,
  ) -> ActorResult<()> {
    self.pre_restart(ctx, reason, msg)
  }
  fn around_pre_start(&mut self, ctx: ActorContextRef<Self::Msg>) -> ActorResult<()> {
    self.pre_start(ctx)
  }
  fn pre_start(&mut self, _ctx: ActorContextRef<Self::Msg>) -> ActorResult<()> {
    Ok(())
  }

  fn pre_restart(
    &mut self,
    _ctx: ActorContextRef<Self::Msg>,
    _reason: ActorError,
    _msg: Option<Self::Msg>,
  ) -> ActorResult<()> {
    Ok(())
  }
  fn pre_suspend(&mut self, _ctx: ActorContextRef<Self::Msg>) {}
  fn post_resume(&mut self, _ctx: ActorContextRef<Self::Msg>, _caused_by_failure: Option<ActorError>) {}
  fn post_stop(&mut self, _ctx: ActorContextRef<Self::Msg>) {}
}

pub trait ActorImmutableBehavior: Debug {
  type Msg: Message;
  fn receive(&self, ctx: ActorContextRef<Self::Msg>, msg: Self::Msg)
    -> Rc<dyn ActorImmutableBehavior<Msg = Self::Msg>>;
}

#[derive(Debug, Clone)]
pub enum Actor<Msg: Message> {
  Mutable(Rc<RefCell<dyn ActorMutableBehavior<Msg = Msg>>>),
  Immutable(Rc<dyn ActorImmutableBehavior<Msg = Msg>>),
}

impl<Msg: Message> Actor<Msg> {
  pub fn of_mutable(behavior: Rc<RefCell<dyn ActorMutableBehavior<Msg = Msg>>>) -> Self {
    Actor::Mutable(behavior)
  }

  pub fn of_immutable(behavior: Rc<dyn ActorImmutableBehavior<Msg = Msg>>) -> Self {
    Actor::Immutable(behavior)
  }
}

impl<Msg: Message> ActorMutableBehavior for Actor<Msg> {
  type Msg = Msg;

  fn receive(&mut self, ctx: ActorContextRef<Self::Msg>, msg: Self::Msg) -> ActorResult<()> {
    match self {
      Self::Mutable(actor) => actor.borrow_mut().receive(ctx, msg),
      Self::Immutable(actor) => {
        let new_actor = actor.as_ref().receive(ctx, msg);
        *self = Self::Immutable(new_actor);
        Ok(())
      }
    }
  }

  fn pre_start(&mut self, ctx: ActorContextRef<Self::Msg>) -> ActorResult<()> {
    match self {
      Self::Mutable(actor) => {
        log::debug!("ActorHandle::pre_start");
        actor.borrow_mut().pre_start(ctx)
      }
      Self::Immutable(_actor) => Ok(()),
    }
  }
}
