use crate::actor::actor_cell::ActorCell;

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
