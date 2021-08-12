use crate::kernel::{Envelope, Message};
use std::sync::mpsc::Receiver;

pub struct QueueReader<Msg: Message> {
  rx: Receiver<Envelope<Msg>>,
}

pub struct QueueEmpty;
pub type DequeueResult<Msg> = Result<Msg, QueueEmpty>;

impl<Msg: Message> QueueReader<Msg> {
  pub fn new(rx: Receiver<Envelope<Msg>>) -> Self {
    Self { rx }
  }

  pub fn dequeue(&self) -> Envelope<Msg> {
    self.rx.recv().unwrap()
  }

  pub fn try_dequeue(&self) -> DequeueResult<Envelope<Msg>> {
    self.rx.try_recv().map_err(|_| QueueEmpty)
  }

  pub fn non_empty(&self) -> bool {
    match self.rx.try_recv() {
      Ok(_) => true,
      Err(_) => false,
    }
  }

  pub fn is_empty(&self) -> bool {
    !self.non_empty()
  }
}
