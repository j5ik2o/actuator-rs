use std::fmt::{Debug};
use std::sync::Arc;
use crate::kernel::system_message::SystemMessage;

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
}

#[derive(Debug, Clone)]
pub struct Envelope {
  message: Arc<dyn AnyMessage>,
  sender: Arc<dyn ActorRef>,
}
