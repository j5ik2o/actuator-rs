#[allow(dead_code)]
mod queue_reader;
mod queue_writer;

use crate::kernel::{Envelope, Message};
pub use queue_reader::*;
pub use queue_writer::*;
use std::sync::mpsc::*;

pub(crate) fn new_queue<Msg: Message>() -> (QueueWriter<Msg>, QueueReader<Msg>) {
  let (tx, rx) = channel::<Envelope<Msg>>();
  let qw = QueueWriter::new(tx);
  let qr = QueueReader::new(rx);
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
    let (qw, qr) = new_queue();
    let expected_message = Envelope::new(Counter(1));
    qw.try_enqueue(expected_message.clone()).unwrap();

    let received_message = qr.try_dequeue().unwrap_or_default().unwrap();
    assert_eq!(received_message, expected_message)
  }
}
