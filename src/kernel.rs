use std::fmt::Debug;
use std::sync::Arc;

mod mailbox;
mod message;
mod system_message;

pub enum MailboxType {
  MPSC,
  VecQueue,
}

pub trait ActorRef: Debug {}

pub trait AnyMessage: Debug + Send {}

pub trait ActorCell {
  fn invoke(&mut self, msg: &Envelope);
}

#[derive(Debug, Clone)]
pub struct Envelope {
  message: Arc<dyn AnyMessage>,
  sender: Arc<dyn ActorRef>,
}
