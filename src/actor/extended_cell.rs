use crate::actor::actor_cell::ActorCell;
use crate::kernel::{MailboxSender, Message, new_mailbox, MailboxType};

#[derive(Debug, Clone)]
pub struct ExtendedCell<M: Message> {
  cell: ActorCell,
  pub(crate) mailbox_sender: MailboxSender<M>,
}

impl<M: Message> Default for ExtendedCell<M> {
  fn default() -> Self {
    let mailbox = new_mailbox(MailboxType::MPSC, 1);
    let mailbox_sender = mailbox.new_sender();
    Self {
      cell: ActorCell::default(),
      mailbox_sender: mailbox_sender,
    }
  }
}

impl<M: Message> ExtendedCell<M> {
  pub fn new(cell: ActorCell, mailbox: MailboxSender<M>) -> Self {
    Self {
      cell,
      mailbox_sender: mailbox,
    }
  }

  // pub(crate) fn send_msg(&self, msg: Envelope<M>) -> MsgResult<Envelope<M>> {
  //
  // }
}
