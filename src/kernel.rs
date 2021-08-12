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
    let (qw, qr) = new_mailbox(1);
    let expected_message = Envelope::new(Counter(1));
    qw.try_enqueue(expected_message.clone()).unwrap();

    let received_message = qr.try_dequeue().unwrap_or(Envelope::new(Counter(0)));
    assert_eq!(received_message, expected_message)
  }
}
