use crate::core::actor::{ActorBehavior, AnyMessageActorWrapper, MockActorMutable};
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::message::Message;
use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

pub trait Props<Msg: Message>: Debug {
  type Actor: ActorBehavior<Msg>;
  fn new_actor(&self) -> Self::Actor;
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
  type Actor = MockActorMutable<Msg>;

  fn new_actor(&self) -> Self::Actor {
    MockActorMutable { p: PhantomData }
  }
}

#[derive(Debug, Clone)]
pub struct AnyProps<Msg: Message> {
  pub underlying: Rc<dyn Props<Msg, Actor = AnyMessageActorWrapper<Msg>>>,
}

impl<Msg: Message> AnyProps<Msg> {
  pub fn new(underlying: Rc<dyn Props<Msg, Actor = AnyMessageActorWrapper<Msg>>>) -> Self {
    Self { underlying }
  }
}

impl<Msg: Message> Props<AnyMessage> for AnyProps<Msg> {
  type Actor = AnyMessageActorWrapper<Msg>;

  fn new_actor(&self) -> Self::Actor {
    AnyMessageActorWrapper::new(self.underlying.new_actor())
  }
  // fn new_actor<A: ActorBehavior<Msg>>(&self) -> A {
  //   AnyMessageActorWrapper::new(self.underlying.new_actor())
  // }
}

#[derive(Debug, Clone)]
pub struct SingletonProps<Msg: Message, A: ActorBehavior<Msg> + Clone> {
  actor: A,
}

impl<Msg: Message, A: ActorBehavior<Msg> + Clone> SingletonProps<Msg, A> {
  pub fn new(actor: A) -> Self {
    Self { actor }
  }
}

impl<Msg: Message, A: ActorBehavior<Msg> + Clone> Props<Msg> for SingletonProps<Msg, A> {
  fn new_actor<X: ActorBehavior<Msg>>(&self) -> X {
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
