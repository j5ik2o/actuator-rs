use crate::infrastructure::queue::Element;
use std::any::Any;

use crate::core::actor::actor_ref::ActorRef;
use crate::core::dispatch::any_message::{AnyMessage, DowncastAnyMessageError};
use crate::core::dispatch::message::Message;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Envelope {
  pub(crate) message: AnyMessage,
  sender: Option<ActorRef<AnyMessage>>,
}

impl Envelope {
  pub fn new<T>(message: T) -> Self
  where
    T: Any + Message + 'static, {
    Envelope {
      message: AnyMessage::new(message),
      sender: None,
    }
  }

  pub fn new_with_sender<T>(message: T, sender: ActorRef<AnyMessage>) -> Self
  where
    T: Any + Message + 'static, {
    Envelope {
      message: AnyMessage::new(message),
      sender: Some(sender),
    }
  }

  pub fn untyped_message(&self) -> Rc<dyn Any + Send> {
    self.message.msg.as_ref().unwrap().clone()
  }

  pub fn typed_message<T>(&self) -> Result<T, DowncastAnyMessageError>
  where
    T: Any + Message + 'static, {
    self.message.take::<T>()
  }

  pub fn sender(&self) -> Option<ActorRef<AnyMessage>> {
    self.sender.clone()
  }
}

unsafe impl Send for Envelope {}
unsafe impl Sync for Envelope {}

impl PartialEq for Envelope {
  fn eq(&self, other: &Self) -> bool {
    self.message == other.message
  }
}

impl Element for Envelope {}
