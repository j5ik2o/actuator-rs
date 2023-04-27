mod message;

use crate::message::Message;
use anyhow::Result;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;

pub trait Actor: Debug {
  type M: Message;

  fn around_receive(&mut self, msg: Self::M) -> Result<()> {
    self.receive(msg)
  }

  fn receive(&mut self, msg: Self::M) -> Result<()>;
}

pub trait Props {
  type A: Actor;
  fn new_actor(&self) -> Self::A;
}

pub struct ActorCell<A: Actor> {
  props: Rc<dyn Props<A = A>>,
  actor: Rc<RefCell<Option<A>>>,
}

impl<A: Actor> ActorCell<A> {
  pub fn new(props: Rc<dyn Props<A = A>>) -> Self {
    Self {
      props,
      actor: Rc::new(RefCell::new(None)),
    }
  }

  pub fn new_actor(&self) -> Rc<RefCell<A>> {
    let actor = Rc::new(RefCell::new(self.props.new_actor()));
    let mut b = self.actor.borrow_mut();
    *b = Some(actor.clone());
    actor
  }
}

pub struct FunctionProps<A: Actor> {
  p: PhantomData<A>,
  f: Rc<dyn Fn() -> A>,
}

impl<A: Actor> FunctionProps<A> {
  pub fn new(f: Rc<dyn Fn() -> A>) -> Self {
    Self { p: PhantomData, f }
  }
}

impl<A: Actor> Props for FunctionProps<A> {
  type A = A;

  fn new_actor(&self) -> Self::A {
    (self.f)()
  }
}

#[derive(Debug)]
pub struct MyActor {}

impl Actor for MyActor {
  type M = String;

  fn receive(&mut self, msg: Self::M) -> Result<()> {
    println!("MyActor received: {}", msg);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{Actor, ActorCell, FunctionProps, MyActor};
  use std::rc::Rc;

  #[test]
  fn it_works() {
    let props = FunctionProps::new(Rc::new(|| MyActor {}));
    let cell = ActorCell::<MyActor>::new(Rc::new(props));
    let actor = cell.new_actor();
    actor.borrow().receive("Hello".to_string()).unwrap();
  }
}
