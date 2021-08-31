use crate::actor::ExtendedCell;
use crate::kernel::message::Message;
use crate::kernel::envelope::Envelope;
use crate::kernel::mailbox_sender::MailboxSender;

pub struct Dispatcher<M: Message> {
  msg: M,
}

impl<M: Message> Dispatcher<M> {
  fn register_for_execution(&self, _mbox: MailboxSender<M>) {}

  pub fn dispatch(&self, receiver: ExtendedCell<M>, invocation: Envelope<M>) {
    let mailbox_sender = receiver.mailbox_sender.clone();
    mailbox_sender.try_enqueue(invocation).unwrap();
    self.register_for_execution(mailbox_sender);
  }
}
