use crate::core::actor::actor_cell::ActorCell;
use crate::core::actor::actor_path::ActorPath;
use crate::core::dispatch::any_message::AnyMessage;

use crate::core::dispatch::message::Message;
use crate::core::dispatch::system_message::system_message_entry::SystemMessageEntry;

use crate::core::actor::actor_ref::dead_letters_ref::DeadLettersRef;
use crate::core::actor::actor_ref::local_actor_ref::LocalActorRef;
use crate::core::actor::ActorError;

pub mod dead_letters_ref;
pub mod local_actor_ref;

pub trait AnyActorRefBehavior {
  fn tell_any(&self, msg: AnyMessage);
}

pub trait ActorRefBehavior<Msg: Message> {
  fn to_any(self) -> AnyActorRef;
  fn path(&self) -> ActorPath;
  fn tell(&mut self, msg: Msg);
  fn send_system_message(&mut self, message: &mut SystemMessageEntry);
}

pub trait ActorRefInnerBehavior<Msg: Message> {
  fn to_any(self) -> AnyActorRef;
  fn path(&self) -> ActorPath;
  fn tell(&mut self, self_ref: ActorRef<Msg>, msg: Msg);
  fn send_system_message(&mut self, self_ref: ActorRef<Msg>, message: &mut SystemMessageEntry);
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActorRef<Msg: Message> {
  NoSender,
  Local(LocalActorRef<Msg>),
  DeadLetters(DeadLettersRef),
  Mock(ActorPath),
}

unsafe impl<Msg: Message> Send for ActorRef<Msg> {}
unsafe impl<Msg: Message> Sync for ActorRef<Msg> {}

pub type AnyActorRef = ActorRef<AnyMessage>;
pub type AnyLocalActorRef = LocalActorRef<AnyMessage>;

impl<Msg: Message> ActorRefBehavior<Msg> for ActorRef<Msg> {
  fn to_any(self) -> AnyActorRef {
    match self {
      ActorRef::NoSender => ActorRef::NoSender,
      ActorRef::Local(local_ref) => ActorRef::Local(local_ref.to_any()),
      ActorRef::DeadLetters(dead_letters_ref) => ActorRef::DeadLetters(dead_letters_ref),
      ActorRef::Mock(path) => ActorRef::Mock(path),
    }
  }

  fn path(&self) -> ActorPath {
    match self {
      ActorRef::NoSender => panic!("NoSender has no path"),
      ActorRef::Local(local_ref) => local_ref.path(),
      ActorRef::DeadLetters(dead_letters_ref) => dead_letters_ref.path(),
      ActorRef::Mock(path) => path.clone(),
    }
  }

  fn tell(&mut self, msg: Msg) {
    let cloned_self = self.clone();
    match self {
      ActorRef::NoSender => {}
      ActorRef::Local(local_ref) => local_ref.tell(cloned_self, msg),
      ActorRef::DeadLetters(dead_letters_ref) => {
        let any_message = AnyMessage::new(msg);
        dead_letters_ref.tell(cloned_self.to_any(), any_message)
      }
      ActorRef::Mock(_) => {}
    }
  }

  fn send_system_message(&mut self, message: &mut SystemMessageEntry) {
    let cloned_self = self.clone();
    match self {
      ActorRef::NoSender => {}
      ActorRef::Local(local_ref) => local_ref.send_system_message(cloned_self, message),
      ActorRef::DeadLetters(dead_letters_ref) => dead_letters_ref.send_system_message(cloned_self.to_any(), message),
      ActorRef::Mock(_) => {}
    }
  }
}

impl<Msg: Message> ActorRef<Msg> {
  pub fn of_no_sender() -> Self {
    ActorRef::NoSender
  }

  pub fn of_local(actor_cell: ActorCell<Msg>, path: ActorPath) -> Self {
    ActorRef::Local(LocalActorRef::new(actor_cell, path)) // , actor_cell))
  }

  pub fn of_dead_letters(path: ActorPath) -> Self {
    ActorRef::DeadLetters(DeadLettersRef::new(path))
  }

  pub fn of_mock(path: ActorPath) -> Self {
    ActorRef::Mock(path)
  }

  pub fn as_local(&self) -> Option<&LocalActorRef<Msg>> {
    match self {
      ActorRef::Local(local_ref) => Some(local_ref),
      _ => None,
    }
  }

  pub fn as_local_mut(&mut self) -> Option<&mut LocalActorRef<Msg>> {
    match self {
      ActorRef::Local(local_ref) => Some(local_ref),
      _ => None,
    }
  }

  pub fn start(&mut self) {
    let cloned_self = self.clone();
    match self {
      ActorRef::Local(local_ref) => local_ref.start(cloned_self),
      _ => {}
    }
  }

  pub fn stop(&mut self) {
    let cloned_self = self.clone();
    match self {
      ActorRef::Local(local_ref) => local_ref.stop(cloned_self),
      _ => {}
    }
  }

  pub fn suspend(&mut self) {
    let cloned_self = self.clone();
    match self {
      ActorRef::Local(local_ref) => local_ref.suspend(cloned_self),
      _ => {}
    }
  }

  pub fn resume(&mut self, caused_by_failure: Option<ActorError>) {
    let cloned_self = self.clone();
    match self {
      ActorRef::Local(local_ref) => local_ref.resume(cloned_self, caused_by_failure),
      _ => {}
    }
  }
}

impl ActorRef<AnyMessage> {
  pub fn to_typed<Msg: Message>(self) -> ActorRef<Msg> {
    match self {
      ActorRef::NoSender => ActorRef::NoSender,
      ActorRef::Local(local_ref) => ActorRef::Local(local_ref.to_typed()),
      ActorRef::DeadLetters(dead_letters_ref) => ActorRef::DeadLetters(dead_letters_ref),
      ActorRef::Mock(path) => ActorRef::Mock(path),
    }
  }
}
