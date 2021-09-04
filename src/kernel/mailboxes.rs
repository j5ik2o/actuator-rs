use crate::kernel::mailbox::Mailbox;
use crate::kernel::message::Message;

pub struct DefaultMailboxes<M: Message> {
  mailboxes: Vec<Mailbox<M>>,
}

pub trait Mailboxes {}

impl<M: Message> Mailboxes for DefaultMailboxes<M> {}
