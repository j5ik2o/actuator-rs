pub mod actor_cell;
pub mod actor_cell_with_ref;
pub mod actor_context;
pub mod actor_path;
pub mod actor_ref;
pub mod actor_ref_provider;
pub mod actor_system;
pub mod address;
pub mod children;
pub mod children_refs;
pub mod props;
pub mod scheduler;

use crate::core::actor::actor_context::ActorContext;
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

pub trait ActorMutableBehavior<Msg: Message>: Debug {
  fn around_receive(&mut self, ctx: ActorContext<Msg>, msg: Msg) -> ActorResult<()> {
    self.receive(ctx, msg)
  }

  fn receive(&mut self, ctx: ActorContext<Msg>, msg: Msg) -> ActorResult<()>;

  fn around_pre_restart(&mut self, ctx: ActorContext<Msg>, reason: ActorError, msg: Option<Msg>) -> ActorResult<()> {
    self.pre_restart(ctx, reason, msg)
  }

  fn pre_restart(&mut self, _ctx: ActorContext<Msg>, _reason: ActorError, _msg: Option<Msg>) -> ActorResult<()> {
    Ok(())
  }

  fn around_pre_start(&mut self, ctx: ActorContext<Msg>) -> ActorResult<()> {
    self.pre_start(ctx)
  }

  fn pre_start(&mut self, _ctx: ActorContext<Msg>) -> ActorResult<()> {
    Ok(())
  }

  fn around_pre_suspend(&mut self, ctx: ActorContext<Msg>) -> ActorResult<()> {
    self.pre_suspend(ctx)
  }

  fn pre_suspend(&mut self, _ctx: ActorContext<Msg>) -> ActorResult<()> {
    Ok(())
  }

  fn around_post_resume(&mut self, ctx: ActorContext<Msg>, caused_by_failure: Option<ActorError>) -> ActorResult<()> {
    self.post_resume(ctx, caused_by_failure)
  }

  fn post_resume(&mut self, _ctx: ActorContext<Msg>, _caused_by_failure: Option<ActorError>) -> ActorResult<()> {
    Ok(())
  }

  fn around_post_stop(&mut self, ctx: ActorContext<Msg>) -> ActorResult<()> {
    self.post_stop(ctx)
  }

  fn post_stop(&mut self, _ctx: ActorContext<Msg>) -> ActorResult<()> {
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct MockActorMutable<Msg: Message> {
  p: PhantomData<Msg>,
}

impl<Msg: Message> ActorMutableBehavior<Msg> for MockActorMutable<Msg> {
  fn receive(&mut self, _ctx: ActorContext<Msg>, _msg: Msg) -> ActorResult<()> {
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct AnyMessageActorWrapper<Msg: Message> {
  actor: Rc<RefCell<dyn ActorMutableBehavior<Msg>>>,
}

impl<Msg: Message> AnyMessageActorWrapper<Msg> {
  pub fn new(actor: Rc<RefCell<dyn ActorMutableBehavior<Msg>>>) -> Self {
    Self { actor }
  }
}

impl<Msg: Message> ActorMutableBehavior<AnyMessage> for AnyMessageActorWrapper<Msg> {
  fn receive(&mut self, ctx: ActorContext<AnyMessage>, msg: AnyMessage) -> ActorResult<()> {
    let typed_msg = msg.take::<Msg>().unwrap();
    let mut actor = self.actor.borrow_mut();
    actor.around_receive(ctx.to_typed(), typed_msg)
  }
}

pub struct AnyMessageActorFunctionWrapper<Msg: Message> {
  actor_f: Rc<dyn Fn() -> Rc<RefCell<dyn ActorMutableBehavior<Msg>>>>,
  actor: Option<Rc<RefCell<dyn ActorMutableBehavior<Msg>>>>,
}

impl<Msg: Message> Debug for AnyMessageActorFunctionWrapper<Msg> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AnyMessageActorFunctionWrapper").finish()
  }
}

impl<Msg: Message> Clone for AnyMessageActorFunctionWrapper<Msg> {
  fn clone(&self) -> Self {
    Self {
      actor_f: self.actor_f.clone(),
      actor: self.actor.clone(),
    }
  }
}

impl<Msg: Message> AnyMessageActorFunctionWrapper<Msg> {
  pub fn new<F>(f: F) -> Self
  where
    F: Fn() -> Rc<RefCell<dyn ActorMutableBehavior<Msg>>> + 'static, {
    Self {
      actor_f: Rc::new(f),
      actor: None,
    }
  }
}

impl<Msg: Message> ActorMutableBehavior<AnyMessage> for AnyMessageActorFunctionWrapper<Msg> {
  fn pre_start(&mut self, _ctx: ActorContext<AnyMessage>) -> ActorResult<()> {
    let a = (self.actor_f)();
    self.actor = Some(a);
    Ok(())
  }

  fn receive(&mut self, ctx: ActorContext<AnyMessage>, msg: AnyMessage) -> ActorResult<()> {
    let typed_msg = msg.take::<Msg>().unwrap();
    let mut actor = self.actor.as_mut().unwrap().borrow_mut();
    actor.around_receive(ctx.to_typed(), typed_msg)
  }
}
