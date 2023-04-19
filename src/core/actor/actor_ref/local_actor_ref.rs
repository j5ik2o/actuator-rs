use crate::core::actor::actor_cell::ActorCell;
use crate::core::actor::actor_path::ActorPath;
use crate::core::actor::actor_ref::ActorRef;
use crate::core::actor::ActorError;
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::message::Message;
use crate::core::dispatch::system_message::system_message_entry::SystemMessageEntry;

#[derive(Debug, Clone)]
pub struct LocalActorRef<Msg: Message> {
  _phantom: std::marker::PhantomData<Msg>,
  actor_cell: ActorCell<Msg>,
  path: ActorPath,
}

impl<Msg: Message> PartialEq for LocalActorRef<Msg> {
  fn eq(&self, other: &Self) -> bool {
    self.path == other.path
  }
}

impl<Msg: Message> LocalActorRef<Msg> {
  pub fn new(actor_cell: ActorCell<Msg>, path: ActorPath) -> Self {
    Self {
      _phantom: std::marker::PhantomData,
      actor_cell,
      path,
    }
  }

  pub fn actor_cell(&self) -> ActorCell<Msg> {
    self.actor_cell.clone()
  }

  pub fn to_any(self, validate_actor: bool) -> LocalActorRef<AnyMessage> {
    LocalActorRef::<AnyMessage>::new(self.actor_cell.to_any(validate_actor), self.path)
  }

  pub fn path(&self) -> ActorPath {
    self.path.clone()
  }

  pub fn tell_any(&mut self, self_ref: ActorRef<AnyMessage>, msg: AnyMessage) {
    self.actor_cell.clone().to_any(true).send_message(self_ref, msg);
  }

  pub fn tell(&mut self, self_ref: ActorRef<Msg>, message: Msg) {
    self.actor_cell.send_message(self_ref, message);
  }

  pub fn send_system_message(&mut self, self_ref: ActorRef<Msg>, message: &mut SystemMessageEntry) {
    self.actor_cell.send_system_message(self_ref, message);
  }

  pub fn start(&mut self, self_ref: ActorRef<Msg>) {
    self.actor_cell.start(self_ref);
  }

  pub fn suspend(&mut self, self_ref: ActorRef<Msg>) {
    self.actor_cell.suspend(self_ref);
  }

  pub fn resume(&mut self, self_ref: ActorRef<Msg>, caused_by_failure: Option<ActorError>) {
    self.actor_cell.resume(self_ref, caused_by_failure)
  }

  pub fn stop(&mut self, self_ref: ActorRef<Msg>) {
    self.actor_cell.stop(self_ref);
  }
}

impl LocalActorRef<AnyMessage> {
  pub fn to_typed<Msg: Message>(self, validate_actor: bool) -> LocalActorRef<Msg> {
    LocalActorRef {
      _phantom: std::marker::PhantomData,
      actor_cell: self.actor_cell.to_typed(validate_actor),
      path: self.path,
    }
  }
}
