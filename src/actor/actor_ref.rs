use crate::kernel::Message;
use crate::actor::extended_cell::ExtendedCell;

#[derive(Clone)]
pub struct ActorRef<M: Message> {
  pub cell: ExtendedCell<M>,
}
