use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::actor::actor_ref::UntypedActorRefBehavior;
use crate::dispatch::any_message::AnyMessage;
use crate::dispatch::message::Message;
use crate::infrastructure::queue::Element;
use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Envelope {
  message: AnyMessage,
  sender: Option<ActorRefRef<AnyMessage>>,
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

  pub fn new_with_sender<T>(message: T, sender: ActorRefRef<AnyMessage>) -> Self
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

  pub fn typed_message<T>(&self) -> T
  where
    T: Any + Message + 'static, {
    self.message.take::<T>().unwrap()
  }

  pub fn sender(&self) -> Option<ActorRefRef<AnyMessage>> {
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
