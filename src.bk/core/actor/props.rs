use crate::core::actor::{ActorBehavior, AnyMessageActorWrapper, MockActorMutable};
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::message::Message;
use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

pub trait Props<Msg: Message>: Debug {
  fn new_actor(&self) -> Rc<RefCell<dyn ActorBehavior<Msg>>>;
}

#[derive(Debug, Clone)]
pub struct MockProps<Msg: Message> {
  p: PhantomData<Msg>,
}

impl<Msg: Message> MockProps<Msg> {
  pub fn new() -> Self {
    Self { p: PhantomData }
  }
}

impl<Msg: Message> Props<Msg> for MockProps<Msg> {
  fn new_actor(&self) -> Rc<RefCell<dyn ActorBehavior<Msg>>> {
    Rc::new(RefCell::new(MockActorMutable { p: PhantomData }))
  }
}

#[derive(Debug, Clone)]
pub struct AnyProps<Msg: Message> {
  pub underlying: Rc<dyn Props<Msg>>,
}

impl<Msg: Message> AnyProps<Msg> {
  pub fn new(underlying: Rc<dyn Props<Msg>>) -> Self {
    Self { underlying }
  }
}

impl<Msg: Message> Props<AnyMessage> for AnyProps<Msg> {
  fn new_actor(&self) -> Rc<RefCell<dyn ActorBehavior<AnyMessage>>> {
    Rc::new(RefCell::new(AnyMessageActorWrapper::new(self.underlying.new_actor())))
  }
}

#[derive(Debug, Clone)]
pub struct SingletonProps<Msg: Message> {
  actor: Rc<RefCell<dyn ActorBehavior<Msg>>>,
}

impl<Msg: Message> SingletonProps<Msg> {
  pub fn new(actor: Rc<RefCell<dyn ActorBehavior<Msg>>>) -> Self {
    Self { actor }
  }
}

impl<Msg: Message> Props<Msg> for SingletonProps<Msg> {
  fn new_actor(&self) -> Rc<RefCell<dyn ActorBehavior<Msg>>> {
    self.actor.clone()
  }
}

pub struct FunctionProps<Msg: Message> {
  actor_f: Rc<dyn Fn() -> Rc<RefCell<dyn ActorBehavior<Msg>>>>,
}

impl<Msg: Message> Clone for FunctionProps<Msg> {
  fn clone(&self) -> Self {
    Self {
      actor_f: self.actor_f.clone(),
    }
  }
}

impl<Msg: Message> Debug for FunctionProps<Msg> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("FunctionProps").finish()
  }
}

impl<Msg: Message> FunctionProps<Msg> {
  pub fn new<F>(actor_f: F) -> Self
  where
    F: Fn() -> Rc<RefCell<dyn ActorBehavior<Msg>>> + 'static, {
    Self {
      actor_f: Rc::new(actor_f),
    }
  }
}

impl<Msg: Message> Props<Msg> for FunctionProps<Msg> {
  fn new_actor(&self) -> Rc<RefCell<dyn ActorBehavior<Msg>>> {
    (*self.actor_f.clone())()
  }
}
