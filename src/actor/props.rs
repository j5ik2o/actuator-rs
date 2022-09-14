use std::fmt::Debug;
use std::rc::Rc;

use crate::actor::actor::Actor;
use crate::dispatch::message::Message;

pub trait Props<Msg: Message>: Debug {
  fn new_actor(&self) -> Actor<Msg>;
}

#[derive(Debug, Clone)]
pub struct SingletonProps<Msg: Message> {
  actor: Actor<Msg>,
}

impl<Msg: Message> SingletonProps<Msg> {
  pub fn new(actor: Actor<Msg>) -> Self {
    Self { actor }
  }
}

impl<Msg: Message> Props<Msg> for SingletonProps<Msg> {
  fn new_actor(&self) -> Actor<Msg> {
    self.actor.clone()
  }
}

pub struct FunctionProps<Msg: Message> {
  actor_f: Rc<dyn Fn() -> Actor<Msg>>,
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
    F: Fn() -> Actor<Msg> + 'static, {
    Self {
      actor_f: Rc::new(actor_f),
    }
  }
}

impl<Msg: Message> Props<Msg> for FunctionProps<Msg> {
  fn new_actor(&self) -> Actor<Msg> {
    (self.actor_f)()
  }
}
