use crate::actor::actor_context::ActorContextRef;
use crate::dispatch::message::Message;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum ActorError {
  ActorFailed { message: String },
}

pub trait ActorMutableBehavior: Debug {
  type Msg: Message;
  fn receive(&mut self, ctx: ActorContextRef<Self::Msg>, msg: Self::Msg);
  fn pre_start(&mut self, _ctx: ActorContextRef<Self::Msg>) {}

  fn pre_restart(&mut self, _ctx: ActorContextRef<Self::Msg>, _cause: ActorError) {}
  fn pre_suspend(&mut self, _ctx: ActorContextRef<Self::Msg>) {}
  fn post_resume(&mut self, _ctx: ActorContextRef<Self::Msg>, _caused_by_failure: ActorError) {}
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

  fn receive(&mut self, ctx: ActorContextRef<Self::Msg>, msg: Self::Msg) {
    match self {
      Self::Mutable(actor) => actor.borrow_mut().receive(ctx, msg),
      Self::Immutable(actor) => {
        let new_actor = actor.as_ref().receive(ctx, msg);
        *self = Self::Immutable(new_actor);
      }
    }
  }

  fn pre_start(&mut self, ctx: ActorContextRef<Self::Msg>) {
    match self {
      Self::Mutable(actor) => {
        log::debug!("ActorHandle::pre_start");
        actor.borrow_mut().pre_start(ctx);
      }
      Self::Immutable(_actor) => {}
    }
  }
}
