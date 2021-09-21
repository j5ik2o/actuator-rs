use std::cmp::Ordering;
use std::sync::Arc;

use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::{ActorRef, ToActorRef, ToUntypedActorRef, TypedActorRef, UntypedActorRef};
use crate::actor::actor_ref::untyped_actor_ref::Sender;
use crate::actor::ExtendedCell;
use crate::kernel::any_message::AnyMessage;
use crate::kernel::any_message_sender::AnyMessageSender;
use crate::kernel::envelope::Envelope;
use crate::kernel::message::Message;

#[derive(Debug, Clone)]
pub struct DefaultTypedActorRef<M: Message> {
  extended_cell: ExtendedCell<M>,
}

impl<M: Message> DefaultTypedActorRef<M> {
  pub fn new(extended_cell: ExtendedCell<M>) -> DefaultTypedActorRef<M> {
    Self { extended_cell }
  }

  pub fn extended_cell(&self) -> &ExtendedCell<M> {
    &self.extended_cell
  }
}

impl<M: Message> ToActorRef for DefaultTypedActorRef<M> {
  fn to_actor_ref<'a>(self: Arc<Self>) -> Arc<dyn ActorRef + 'a>
  where
    Self: 'a,
  {
    self
  }
}

impl<M: Message> ToUntypedActorRef for DefaultTypedActorRef<M> {
  fn to_untyped_actor_ref<'a>(self: Arc<Self>) -> Arc<dyn UntypedActorRef + 'a>
  where
    Self: 'a,
  {
    self
  }
}

impl<M: Message> ActorRef for DefaultTypedActorRef<M> {
  fn name(&self) -> &str {
    self.path().name()
  }

  fn path(&self) -> &ActorPath {
    todo!()
    // self.extended_cell.actor_cell().path()
  }
}

impl<M: Message> UntypedActorRef for DefaultTypedActorRef<M> {
  fn tell(self: Arc<Self>, msg: AnyMessage, sender: Sender) {
    let s = self.clone();
    self
      .extended_cell
      .mailbox_sender()
      .try_enqueue_any(s, msg, sender)
      .unwrap();
  }
}

impl<M: Message> TypedActorRef<M> for DefaultTypedActorRef<M> {
  fn tell(self: Arc<Self>, message: M) {
    let envelope = Envelope::new(message, None);
    let s = self.clone();
    self
      .extended_cell
      .mailbox_sender()
      .try_enqueue(s, envelope)
      .unwrap();
  }
}

impl<M: Message> PartialOrd for DefaultTypedActorRef<M> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    let x = self.path().partial_cmp(other.path());
    if x == Some(Ordering::Equal) {
      if self.path().uid() < other.path().uid() {
        Some(Ordering::Less)
      } else if self.path().uid() == other.path().uid() {
        Some(Ordering::Equal)
      } else {
        Some(Ordering::Greater)
      }
    } else {
      x
    }
  }
}

impl<M: Message> PartialEq for DefaultTypedActorRef<M> {
  fn eq(&self, other: &Self) -> bool {
    self.partial_cmp(other) == Some(Ordering::Equal)
  }
}
