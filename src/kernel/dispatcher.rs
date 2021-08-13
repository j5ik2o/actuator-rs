

use crate::kernel::{Envelope, Message, MailboxSender};

use crate::actor::ExtendedCell;


pub struct Dispatcher<M: Message> {
  msg: M,
}

impl<M: Message> Dispatcher<M> {
  fn register_for_execution(&self, _mbox: MailboxSender<M>) {}

  pub fn dispatch(&self, receiver: ExtendedCell<M>, invocation: Envelope<M>) {
    let mbox = receiver.mailbox;
    mbox
      .try_enqueue(ExtendedCell::default(), invocation)
      .unwrap();
    self.register_for_execution(mbox);
  }
}
