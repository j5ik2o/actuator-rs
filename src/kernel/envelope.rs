use crate::kernel::message::Message;
use crate::actor::actor_ref::internal_actor_ref::Sender;

#[derive(Debug, Clone)]
pub struct Envelope<M: Message> {
  pub message: M,
  pub sender: Sender,
}

impl<M: Message> Envelope<M> {
  pub fn new(value: M, sender: Sender) -> Self {
    Self {
      message: value,
      sender,
    }
  }
}
