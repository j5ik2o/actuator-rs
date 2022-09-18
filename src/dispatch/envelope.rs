use crate::actor::actor_ref::ActorRef;
use crate::dispatch::any_message::AnyMessage;
use std::any::Any;
use std::sync::Arc;

pub struct Envelope {
  message: AnyMessage,
  sender: ActorRef,
}

impl Envelope {
  pub fn new(message: Option<AnyMessage>, sender: ActorRef /* system: ActorSystem */) -> Self {
    if message.is_none() {
      if sender == ActorRef::NoSender {
        panic!("Message is None");
      } else {
        panic!("Message sent from [{}] is None.", sender);
      }
    }
    Self {
      message: message.unwrap(),
      sender: if sender != ActorRef::NoSender {
        sender
      } else {
        /// deadletter
        sender
      },
    }
  }

  pub fn copy(self, message: Option<AnyMessage>, sender: Option<ActorRef>) -> Self {
    let mut result = self;
    if let Some(m) = message {
      result.message = m;
    }
    if let Some(s) = sender {
      result.sender = s;
    }
    result
  }
}
