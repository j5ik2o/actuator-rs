use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::mpsc::*;

use anyhow::Result;

pub use mpsc::*;

use crate::kernel::envelope::Envelope;
use crate::kernel::message::Message;
use crate::kernel::queue::vec_deque::QueueInVecQueue;

mod mpsc;
mod vec_deque;

pub enum MessageSize {
  Limit(usize),
  Limitless,
}

pub trait QueueReader<M: Message>: Debug {
  fn dequeue(&self) -> Envelope<M>;
  fn dequeue_opt(&self) -> Option<Envelope<M>> {
    self.try_dequeue().unwrap()
  }
  fn try_dequeue(&self) -> Result<Option<Envelope<M>>>;
  fn non_empty(&self) -> bool;
  fn is_empty(&self) -> bool {
    !self.non_empty()
  }
  fn number_of_messages(&self) -> MessageSize;
}

pub trait QueueWriter<M: Message>: Debug {
  fn try_enqueue(&self, msg: Envelope<M>) -> Result<()>;
}

pub(crate) fn new_vec_queue<M: Message>() -> (impl QueueWriter<M>, impl QueueReader<M>) {
  let vec_q = VecDeque::new();
  let q = QueueInVecQueue::new(vec_q);
  (q.clone(), q.clone())
}

pub(crate) fn new_mpsc_queue<M: Message>() -> (impl QueueWriter<M>, impl QueueReader<M>) {
  let (tx, rx) = channel::<Envelope<M>>();
  let qw = QueueWriterInMPSC::new(tx);
  let qr = QueueReaderInMPSC::new(rx);
  (qw, qr)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Clone, PartialEq)]
  struct Counter(u32);

  //  impl Message for Counter {}

  #[test]
  fn test_new_vec_queue() {
    let (qw, qr) = new_vec_queue();
    let expected_message = Envelope::new(Counter(1), None);
    qw.try_enqueue(expected_message.clone()).unwrap();

    match qr.number_of_messages() {
      MessageSize::Limit(n) => assert!(n > 0),
      MessageSize::Limitless => panic!(),
    }
    let received_message = qr.try_dequeue().unwrap_or_default().unwrap();
    assert_eq!(received_message.message, expected_message.message)
  }

  #[test]
  fn test_new_mpsc_queue() {
    let (qw, qr) = new_mpsc_queue();
    let expected_message = Envelope::new(Counter(1), None);
    qw.try_enqueue(expected_message.clone()).unwrap();
    let received_message = qr.try_dequeue().unwrap_or_default().unwrap();
    assert_eq!(received_message.message, expected_message.message)
  }
}
