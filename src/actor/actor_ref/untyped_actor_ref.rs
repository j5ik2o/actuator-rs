use crate::actor::actor_cell::ActorCell;

#[derive(Debug, Clone)]
pub struct UntypedActorRef {
  pub(crate) cell: ActorCell,
}

pub type Sender = Option<UntypedActorRef>;



impl UntypedActorRef {
  pub fn new(cell: ActorCell) -> Self {
    Self { cell }
  }
}
