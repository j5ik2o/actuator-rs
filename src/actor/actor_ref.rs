use std::fmt::{Display, Formatter};

use crate::actor::actor_cell;
use crate::actor::actor_path::{ActorPath, ActorPathBehavior};
use crate::dispatch::any_message::AnyMessage;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ActorRef {
  NoSender,
  Local(LocalActorRef),
}

impl Display for ActorRef {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if self.path().uid() == actor_cell::UNDEFINED_UID {
      write!(f, "Actor[{}]", self.path())
    } else {
      write!(f, "Actor[{}#{}]", self.path(), self.path().uid())
    }
  }
}

pub trait ActorRefBehavior: PartialOrd {
  type Message;
  fn path(&self) -> &ActorPath;
  fn tell(&self, msg: Self::Message);
}

pub trait InternalActorRefBehavior: ActorRefBehavior {
  fn start(&self);
  fn resume(&self);
  fn suspend(&self);
  fn restart<F>(&self, panic_f: F)
  where
    F: Fn();
  fn stop(&self);
  //   fn send_system_message
}

impl ActorRefBehavior for ActorRef {
  type Message = AnyMessage;

  fn path(&self) -> &ActorPath {
    match self {
      ActorRef::Local(inner) => &inner.path,
      _ => panic!(""),
    }
  }

  fn tell(&self, _msg: Self::Message) {
    todo!()
  }
}

impl InternalActorRefBehavior for ActorRef {
  fn start(&self) {
    todo!()
  }

  fn resume(&self) {
    todo!()
  }

  fn suspend(&self) {
    todo!()
  }

  fn restart<F>(&self, panic_f: F)
  where
    F: Fn(), {
    todo!()
  }

  fn stop(&self) {
    todo!()
  }
}

impl ActorRef {
  pub fn of_local(path: ActorPath) -> Self {
    ActorRef::Local(LocalActorRef::new(path))
  }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LocalActorRef {
  path: ActorPath,
}

impl LocalActorRef {
  pub fn new(path: ActorPath) -> Self {
    Self { path }
  }
}
