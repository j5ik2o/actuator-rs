use std::fmt::{Debug};
use std::sync::{Arc, Mutex};
use crate::kernel::system_message::SystemMessage;
use crate::kernel::mailbox::SystemMessageQueue;

mod mailbox;
mod message;
mod system_message;

pub enum MailboxType {
  MPSC,
  VecQueue,
}

pub trait ActorRef: Debug + Sync + Send {}

#[derive(Debug, Clone)]
pub struct DummyActorRef;

impl ActorRef for DummyActorRef {}

pub trait AnyMessage: Debug + Send {}

pub trait ActorCell {
  fn my_self(&self) -> Arc<dyn ActorRef>;
  fn invoke(&mut self, msg: &Envelope);
  fn system_invoke(&mut self, msg: SystemMessage);
  fn dead_letter_mailbox(&self) -> Arc<Mutex<dyn SystemMessageQueue>>;
}

pub struct DummyActorCell {
  my_self: Arc<dyn ActorRef>,
  dead_letter_mailbox: Arc<Mutex<dyn SystemMessageQueue>>,
}

impl DummyActorCell {
  pub fn new(
    my_self: Arc<dyn ActorRef>,
    dead_letter_mailbox: Arc<Mutex<dyn SystemMessageQueue>>,
  ) -> Self {
    Self {
      my_self,
      dead_letter_mailbox,
    }
  }
}

impl ActorCell for DummyActorCell {
  fn my_self(&self) -> Arc<dyn ActorRef> {
    self.my_self.clone()
  }

  fn invoke(&mut self, _msg: &Envelope) {
    todo!()
  }

  fn system_invoke(&mut self, _msg: SystemMessage) {
    todo!()
  }

  fn dead_letter_mailbox(&self) -> Arc<Mutex<dyn SystemMessageQueue>> {
    self.dead_letter_mailbox.clone()
  }
}

#[derive(Debug, Clone)]
pub struct Envelope {
  message: Arc<dyn AnyMessage>,
  sender: Arc<dyn ActorRef>,
}
