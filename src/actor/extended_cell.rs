use crate::kernel::{Message, Dispatcher};
use crate::actor::actor_cell::ActorCell;

#[derive(Clone)]
pub struct ExtendedCell<M: Message> {
  cell: ActorCell,
  dispatcher: Dispatcher<M>,
}
