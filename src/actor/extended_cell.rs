use crate::kernel::{Message, Dispatcher, Envelope};
use crate::actor::actor_cell::ActorCell;

#[derive(Clone)]
pub struct ExtendedCell<M: Message> {
  cell: ActorCell,
  dispatcher: Dispatcher<M>,
}

#[derive(Clone)]
pub struct MsgError<T> {
  pub msg: T,
}

pub type MsgResult<T> = Result<(), MsgError<T>>;

impl<M: Message> ExtendedCell<M> {
  pub fn new(cell: ActorCell, dispatcher: Dispatcher<M>) -> Self {
    Self { cell, dispatcher }
  }

  // pub(crate) fn send_msg(&self, msg: Envelope<M>) -> MsgResult<Envelope<M>> {
  //
  // }
}
