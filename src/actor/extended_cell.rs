use crate::actor::actor_cell::ActorCell;
use crate::kernel::mailbox::MailboxSender;
use crate::kernel::message::Message;

#[derive(Clone)]
pub struct ExtendedCell<M: Message> {
  cell: ActorCell,
  pub(crate) mailbox_sender: MailboxSender<M>,
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
