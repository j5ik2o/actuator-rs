use crate::actor::actor_cell::ActorCell;
use crate::actor::actor_ref::ActorRef;
use crate::kernel::message::Message;

#[derive(Debug, Clone)]
pub struct InternalActorRef {
  pub(crate) cell: ActorCell,
}

pub type Sender = Option<InternalActorRef>;

impl InternalActorRef {
  pub fn new(cell: ActorCell) -> Self {
    Self { cell }
  }
}
