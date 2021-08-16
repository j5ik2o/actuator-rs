use std::sync::mpsc::*;

pub use reader_mpsc::*;
pub use writer_mpsc::*;

use crate::kernel::{Envelope, Message};
use anyhow::Result;

mod reader_mpsc;
mod writer_mpsc;

pub trait QueueReader<M: Message> {
  fn dequeue(&self) -> Envelope<M>;
  fn dequeue_opt(&self) -> Option<Envelope<M>> {
    self.try_dequeue().unwrap()
  }
  fn try_dequeue(&self) -> Result<Option<Envelope<M>>>;
  fn non_empty(&self) -> bool;
  fn is_empty(&self) -> bool;
  fn number_of_messages(&self) -> usize;
}

pub trait QueueWriter<M: Message> {
  fn try_enqueue(&self, msg: Envelope<M>) -> Result<()>;
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

  impl Message for Counter {}

  #[test]
  fn test_new_queue() {
    let (qw, qr) = new_mpsc_queue();
    let expected_message = Envelope::new(Counter(1));
    qw.try_enqueue(expected_message.clone()).unwrap();

    let received_message = qr.try_dequeue().unwrap_or_default().unwrap();
    assert_eq!(received_message, expected_message)
  }
}
