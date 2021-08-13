use crate::actor::actor_cell::ActorCell;
use crate::kernel::{Message, MailboxSender, new_mailbox};

#[derive(Clone)]
pub struct ExtendedCell<M: Message> {
  cell: ActorCell,
  pub(crate) mailbox: MailboxSender<M>,
}

#[derive(Clone)]
pub struct MsgError<T> {
  pub msg: T,
}

pub type MsgResult<T> = Result<(), MsgError<T>>;

impl<M: Message> Default for ExtendedCell<M> {
  fn default() -> Self {
    let (mailbox, _) = new_mailbox(1);
    Self {
      cell: ActorCell::default(),
      mailbox,
    }
  }
}

impl<M: Message> ExtendedCell<M> {
  pub fn new(cell: ActorCell, mailbox: MailboxSender<M>) -> Self {
    Self { cell, mailbox }
  }

  // pub(crate) fn send_msg(&self, msg: Envelope<M>) -> MsgResult<Envelope<M>> {
  //
  // }
}
