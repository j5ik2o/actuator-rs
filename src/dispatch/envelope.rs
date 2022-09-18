use crate::actor::actor_ref::ActorRef;
use crate::dispatch::any_message::AnyMessage;
use crate::queue::Element;
use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct Envelope {
  message: AnyMessage,
  sender: ActorRef,
}

unsafe impl Send for Envelope {}
unsafe impl Sync for Envelope {}

impl Element for Envelope {}

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
        /// system.deadletter
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

  pub fn message(&self) -> &AnyMessage {
    &self.message
  }

  pub fn message_mut(&mut self) -> &mut AnyMessage {
    &mut self.message
  }

  pub fn sender(&self) -> &ActorRef {
    &self.sender
  }
}
