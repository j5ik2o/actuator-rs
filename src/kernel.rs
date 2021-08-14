use std::fmt::{Debug};
use queue::new_queue;

mod actor_cell;
mod dispatcher;
mod mailbox;
mod queue;

pub use queue::*;
pub use mailbox::*;

pub trait Message: Debug + Clone + Send + Sync + 'static + PartialEq {}

#[derive(Debug, Clone, PartialEq)]
pub struct Envelope<M: Message> {
  pub msg: M,
}

impl<M: Message> Envelope<M> {
  pub fn new(value: M) -> Self {
    Self { msg: value }
  }
}

pub fn new_mailbox<M: Message>(limit: u32) -> Mailbox<M> {
  let (qw, qr) = new_queue();
  let mailbox = Mailbox::new(limit, qr, qw);
  mailbox
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::actor::ExtendedCell;

  #[derive(Debug, Clone, PartialEq)]
  struct Counter(u32);

  impl Message for Counter {}

  #[test]
  fn test_new_mailbox() {
    let mailbox1 = new_mailbox(2);
    let dispatcher1 = mailbox1.new_sender();
    let expected_message1 = Envelope::new(Counter(1));

    dispatcher1
      .try_enqueue(ExtendedCell::default(), expected_message1.clone())
      .unwrap();

    let dispatcher2 = dispatcher1.clone();
    let expected_message2 = Envelope::new(Counter(2));
    dispatcher2
      .try_enqueue(ExtendedCell::default(), expected_message2.clone())
      .unwrap();

    let mut mailbox2 = mailbox1.clone();

    let received_message1 = mailbox2.try_dequeue().unwrap_or_default().unwrap();
    assert_eq!(received_message1, expected_message1);

    let received_message2 = mailbox2.try_dequeue().unwrap_or_default().unwrap();
    assert_eq!(received_message2, expected_message2)
  }
}
