use crate::core::actor::actor_path::ActorPath;
use crate::core::actor::actor_ref::{ActorRef, ActorRefInnerBehavior, AnyActorRef};
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::system_message::system_message_entry::SystemMessageEntry;

#[derive(Debug, Clone, PartialEq)]
pub struct DeadLettersRef {
  path: ActorPath,
}

impl DeadLettersRef {
  pub fn new(path: ActorPath) -> Self {
    Self { path }
  }
}

impl ActorRefInnerBehavior<AnyMessage> for DeadLettersRef {
  fn to_any(self) -> AnyActorRef {
    ActorRef::DeadLetters(self)
  }

  fn path(&self) -> ActorPath {
    self.path.clone()
  }

  fn tell(&mut self, self_ref: ActorRef<AnyMessage>, msg: AnyMessage) {
    log::warn!("DeadLettersRef::tell: self_ref = {:?}, msg = {:?}", self_ref, msg);
  }

  fn send_system_message(&mut self, self_ref: ActorRef<AnyMessage>, message: &mut SystemMessageEntry) {
    log::warn!(
      "DeadLettersRef::send_system_message: self_ref = {:?}, msg = {:?}",
      self_ref,
      message
    );
  }
}
