use crate::kernel::mailbox::Mailbox;
use crate::kernel::message::Message;
use crate::kernel::system_message::SystemMessage;
use crate::kernel::mailbox_sender::MailboxSender;

pub struct DefaultMailboxes<M: Message> {
  mailboxes: Vec<Mailbox<M>>,
  system_mailbox: MailboxSender<SystemMessage>,
}

pub trait Mailboxes {}

impl<M: Message> Mailboxes for DefaultMailboxes<M> {}
