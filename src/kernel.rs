#[allow(dead_code)]
mod actor_cell;
mod dispatcher;
mod mailbox;
mod queue;

pub use dispatcher::Dispatcher;
pub use mailbox::Mailbox;
use queue::new_queue;
use std::fmt::Debug;

pub trait Message: Debug + Clone + Send + 'static + PartialEq {}
// impl<T: Debug + Clone + Send + 'static> Message for T {}

#[derive(Debug, Clone, PartialEq)]
pub struct Envelope<M: Message> {
  //  pub sender: Option<BasicActorRef>,
  pub msg: M,
}

impl<M: Message> Envelope<M> {
  pub fn new(value: M) -> Self {
    Self { msg: value }
  }
}

pub fn new_mailbox<M: Message>(limit: u32) -> (Dispatcher<M>, Mailbox<M>) {
  let (qw, qr) = new_queue();
  let dispatcher = Dispatcher::new(qw);
  let mailbox = Mailbox::new(limit, qr);
  (dispatcher, mailbox)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Clone, PartialEq)]
  struct Counter(u32);

  impl Message for Counter {}

  #[test]
  fn test_new_mailbox() {
    let (dispatcher1, mailbox1) = new_mailbox(1);
    let expected_message1 = Envelope::new(Counter(1));
    dispatcher1.try_enqueue(expected_message1.clone()).unwrap();

    let dispatcher2 = dispatcher1.clone();
    let expected_message2 = Envelope::new(Counter(2));
    dispatcher2.try_enqueue(expected_message2.clone()).unwrap();

    let mailbox2 = mailbox1.clone();

    let received_message1 = mailbox2.try_dequeue().unwrap_or(Envelope::new(Counter(0)));
    assert_eq!(received_message1, expected_message1);

    let received_message2 = mailbox2.try_dequeue().unwrap_or(Envelope::new(Counter(0)));
    assert_eq!(received_message2, expected_message2)
  }
}
