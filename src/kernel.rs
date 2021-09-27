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
  fn invoke(&mut self, msg: &Envelope);
  fn system_invoke(&mut self, msg: SystemMessage);
  fn dead_letter_mailbox(&self) -> Arc<Mutex<dyn SystemMessageQueue>>;
}

pub struct DummyActorCell {
  dead_letter_mailbox: Arc<Mutex<dyn SystemMessageQueue>>,
}

impl DummyActorCell {
  pub fn new(dead_letter_mailbox: Arc<Mutex<dyn SystemMessageQueue>>) -> Self {
    Self {
      dead_letter_mailbox,
    }
  }
}

impl ActorCell for DummyActorCell {
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
