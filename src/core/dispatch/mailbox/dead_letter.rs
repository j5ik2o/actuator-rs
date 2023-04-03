use crate::core::actor::actor_ref::ActorRef;
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::message::Message;

#[derive(Debug, Clone)]
pub struct DeadLetter<Msg: Message> {
  message: AnyMessage,
  sender: ActorRef<Msg>,
  recipient: ActorRef<Msg>,
}

unsafe impl<Msg: Message> Send for DeadLetter<Msg> {}
unsafe impl<Msg: Message> Sync for DeadLetter<Msg> {}

impl<Msg: Message> DeadLetter<Msg> {
  pub fn new(message: AnyMessage, sender: ActorRef<Msg>, recipient: ActorRef<Msg>) -> Self {
    Self {
      message,
      sender,
      recipient,
    }
  }
}

impl<Msg: Message> PartialEq for DeadLetter<Msg> {
  fn eq(&self, other: &Self) -> bool {
    self.message == other.message && self.sender == other.sender && self.recipient == other.recipient
  }
}
