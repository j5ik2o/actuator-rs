use std::sync::Arc;
use crate::actor::ExtendedCell;
use crate::kernel::message::Message;
use crate::actor::actor_ref::{ActorRef, ToActorRef};


use crate::actor::actor_path::ActorPath;
use std::cmp::Ordering;
use crate::kernel::envelope::Envelope;

#[derive(Debug, Clone)]
pub struct TypedActorRef<M: Message> {
  extended_cell: ExtendedCell<M>,
}

impl<M: Message> ToActorRef for TypedActorRef<M> {
  fn to_actor_ref<'a>(self: Arc<Self>) -> Arc<dyn ActorRef + 'a>
  where
    Self: 'a,
  {
    self
  }
}

impl<M: Message> ActorRef for TypedActorRef<M> {
  fn path(&self) -> &ActorPath {
    self.extended_cell.actor_cell().path()
  }
}

impl<M: Message> TypedActorRef<M> {
  pub fn new(extended_cell: ExtendedCell<M>) -> TypedActorRef<M> {
    Self { extended_cell }
  }

  pub fn extended_cell(&self) -> &ExtendedCell<M> {
    &self.extended_cell
  }

  pub fn send_message(&self, message: M) {
    let _envelope = Envelope::new(message, None);
    // let _ = self.cell.send_message(envelope);
  }
}

impl<M: Message> PartialOrd for TypedActorRef<M> {
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

impl<M: Message> PartialEq for TypedActorRef<M> {
  fn eq(&self, other: &Self) -> bool {
    self.partial_cmp(other) == Some(Ordering::Equal)
  }
}
