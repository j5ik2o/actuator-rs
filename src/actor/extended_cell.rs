use crate::actor::actor_cell::ActorCell;
use crate::kernel::message::Message;
use crate::kernel::mailbox_sender::MailboxSender;
use crate::kernel::mailbox::Mailbox;
use std::sync::Arc;
use crate::actor_system::ActorSystem;
use crate::actor::actor_ref::InternalActorRef;
use crate::actor::actor_path::ActorPath;
use crate::kernel::system_message::SystemMessage;

#[derive(Debug, Clone)]
pub struct ExtendedCell<M: Message> {
  actor_cell: ActorCell,
  mailbox_sender: MailboxSender<M>,
}

impl<M: Message> ExtendedCell<M> {
  pub fn new(
    system: Arc<dyn ActorSystem>,
    self_ref: Arc<dyn InternalActorRef>,
    parent_ref: Arc<dyn InternalActorRef>,
    path: ActorPath,
    mailbox: Arc<Mailbox<M>>,
    system_mailbox: MailboxSender<SystemMessage>,
  ) -> Self {
    Self {
      actor_cell: ActorCell::new(
        system,
        self_ref,
        parent_ref,
        path,
        Arc::new(mailbox.new_sender()),
        system_mailbox,
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

  // pub(crate) fn send_msg(&self, msg: Envelope<M>) -> MsgResult<Envelope<M>> {
  //
  // }
}
