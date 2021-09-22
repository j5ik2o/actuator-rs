use std::any::Any;
use std::sync::Arc;
use std::fmt::Debug;

mod mailbox;
mod message;

pub enum MailboxType {
  MPSC,
  VecQueue,
}

pub trait ActorRef: Debug {}

pub trait AnyMessage: Debug + Send {}

#[derive(Debug, Clone)]
pub struct Envelope {
  message: Arc<dyn AnyMessage>,
  sender: Arc<dyn ActorRef>,
}
