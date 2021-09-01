use crate::actor::actor_cell::ActorCell;
use crate::kernel::message::Message;
use crate::kernel::mailbox_sender::MailboxSender;

#[derive(Debug, Clone)]
pub struct ExtendedCell<M: Message> {
  actor_cell: ActorCell,
  mailbox_sender: MailboxSender<M>,
}

impl<M: Message> ExtendedCell<M> {
  pub fn new(actor_cell: ActorCell, mailbox_sender: MailboxSender<M>) -> Self {
    Self {
      actor_cell,
      mailbox_sender,
    }
  }

  pub fn actor_cell(&self) -> &ActorCell {
    &self.actor_cell
  }

  pub fn mailbox_sender(&self) -> &MailboxSender<M> {
    &self.mailbox_sender
  }

  // pub(crate) fn send_msg(&self, msg: Envelope<M>) -> MsgResult<Envelope<M>> {
  //
  // }
}
