use std::cmp::Ordering;
use std::sync::Arc;

use crate::actor::actor_path::ActorPath;
use crate::actor::actor_ref::{ActorRef, ToActorRef, TypedActorRef, UntypedActorRef, ToUntypedActorRef};
use crate::actor::actor_ref::untyped_actor_ref::Sender;
use crate::actor::ExtendedCell;
use crate::kernel::any_message::AnyMessage;
use crate::kernel::envelope::Envelope;
use crate::kernel::message::Message;

#[derive(Debug, Clone)]
pub struct DefaultTypedActorRef<M: Message> {
  extended_cell: ExtendedCell<M>,
}

impl<M: Message> ToActorRef for DefaultTypedActorRef<M> {
  fn to_actor_ref<'a>(self: Arc<Self>) -> Arc<dyn ActorRef + 'a> where Self: 'a {
    self
  }
}

impl<M: Message> ToUntypedActorRef for DefaultTypedActorRef<M> {
  fn to_untyped_actor_ref<'a>(self: Arc<Self>) -> Arc<dyn UntypedActorRef + 'a> where Self: 'a {
    todo!()
  }
}

impl<M: Message> ActorRef for DefaultTypedActorRef<M> {
  fn name(&self) -> &str {
    self.path().name()
  }

  fn path(&self) -> &ActorPath {
    self.extended_cell.actor_cell().path()
  }
}

impl<M: Message> TypedActorRef<M> for DefaultTypedActorRef<M> {
  fn tell(&self, message: M) {
    let envelope = Envelope::new(message, None);
    self
      .extended_cell
      .mailbox_sender()
      .try_enqueue(envelope)
      .unwrap();
  }
}

impl<M: Message> DefaultTypedActorRef<M> {
  pub fn new(extended_cell: ExtendedCell<M>) -> DefaultTypedActorRef<M> {
    Self { extended_cell }
  }

  pub fn extended_cell(&self) -> &ExtendedCell<M> {
    &self.extended_cell
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
