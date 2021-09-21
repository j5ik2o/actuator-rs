use std::sync::Arc;

use crate::actor::actor_cell::ActorCell;

use crate::actor::actor_ref::InternalActorRef;
use crate::actor_system::ActorSystem;
use crate::kernel::envelope::Envelope;
use crate::kernel::mailbox::Mailbox;
use crate::kernel::mailbox_sender::MailboxSender;
use crate::kernel::message::Message;

#[derive(Debug, Clone)]
pub struct ExtendedCell<M: Message> {
  actor_cell: ActorCell,
  mailbox_sender: MailboxSender<M>,
}

impl<M: Message> ExtendedCell<M> {
  pub fn from_actor_cell(actor_cell: ActorCell, mailbox_sender: MailboxSender<M>) -> Self {
    Self {
      actor_cell,
      mailbox_sender,
    }
  }

  pub fn new(
    system: Arc<dyn ActorSystem>,
    _self_ref: Arc<dyn InternalActorRef>,
    _parent_ref: Arc<dyn InternalActorRef>,
    //    path: ActorPath,
    mailbox: Arc<Mailbox<M>>,
    //    system_mailbox: MailboxSender<SystemMessage>,
  ) -> Self {
    Self {
      actor_cell: ActorCell::new(
        system,
        // parent_ref,
        // path,
        // Arc::new(mailbox.new_sender()),
        // system_mailbox,
      ),
      mailbox_sender: mailbox.new_sender(),
    }
  }

  pub fn actor_cell(&self) -> &ActorCell {
    &self.actor_cell
  }

  pub fn mailbox_sender(&self) -> &MailboxSender<M> {
    &self.mailbox_sender
  }

  pub fn invoke(&self, _message: Envelope<M>) {
    todo!()
  }
  // pub(crate) fn send_msg(&self, msg: Envelope<M>) -> MsgResult<Envelope<M>> {
  //
  // }
}
