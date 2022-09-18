use std::fmt::{Display, Formatter};

use crate::actor::actor_cell;
use crate::actor::actor_path::{ActorPath, ActorPathBehavior};
use crate::dispatch::any_message::AnyMessage;

#[derive(PartialOrd)]
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
  type Message;
  fn start(&self);
  fn resume(&self);
  fn suspend(&self);
  fn restart<F>(&self, panic_f: F)
  where
    F: Fn();
  fn stop(&self);
  //   fn send_system_message
}

impl<M> ActorRefBehavior for ActorRef {
  type Message = AnyMessage;

  fn path(&self) -> &ActorPath {
    match self {
      ActorRef::Local(inner) => &inner.path,
      _ => panic!(""),
    }
  }

  fn tell(&self, msg: Self::Message) {
    todo!()
  }
}

impl<M> ActorRef {
  pub fn of_local(path: ActorPath) -> Self {
    ActorRef::Local(LocalActorRef::new(path))
  }
}

pub struct LocalActorRef {
  path: ActorPath,
}

impl LocalActorRef {
  pub fn new(path: ActorPath) -> Self {
    Self { path }
  }
}
