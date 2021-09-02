use anyhow::Result;
use crate::kernel::envelope::Envelope;
use crate::kernel::mailbox::Mailbox;
use crate::kernel::message::Message;

#[derive(Debug, Clone)]
pub struct MailboxSender<M: Message> {
  mailbox: Mailbox<M>,
}

impl<M: Message> MailboxSender<M> {
  pub fn new(mailbox: Mailbox<M>) -> Self {
    Self { mailbox }
  }

  pub fn to_mailbox(&self) -> &Mailbox<M> {
    &self.mailbox
  }

  pub fn try_enqueue(&self, msg: Envelope<M>) -> Result<()> {
    self.mailbox.try_enqueue(msg)
  }

  pub fn set_as_scheduled(&mut self) -> bool {
    self.mailbox.set_as_scheduled()
  }

  pub fn set_as_idle(&mut self) -> bool {
    self.mailbox.set_as_idle()
  }

  pub fn is_scheduled(&self) -> bool {
    self.mailbox.is_scheduled()
  }

  pub fn is_closed(&self) -> bool {
    self.mailbox.is_closed()
  }

  pub fn suspend(&self) -> bool {
    self.mailbox.suspend()
  }

  pub fn resume(&self) -> bool {
    self.mailbox.resume()
  }
}

unsafe impl<M: Message> Send for MailboxSender<M> {}

unsafe impl<M: Message> Sync for MailboxSender<M> {}
