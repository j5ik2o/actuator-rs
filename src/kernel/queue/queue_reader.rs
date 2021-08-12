use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;

use crate::kernel::{Envelope, Message};

pub struct QueueReader<M: Message> {
  inner: Mutex<QueueReaderInner<M>>
}

struct QueueReaderInner<M: Message> {
  rx: Receiver<Envelope<M>>,
}

pub struct QueueEmpty;
pub type DequeueResult<Msg> = Result<Msg, QueueEmpty>;

impl<Msg: Message> QueueReader<Msg> {
  pub fn new(rx: Receiver<Envelope<Msg>>) -> Self {
    Self { inner: Mutex::new(QueueReaderInner { rx }) }
  }

  pub fn dequeue(&self) -> Envelope<Msg> {
    let inner = self.inner.lock().unwrap();
    inner.rx.recv().unwrap()
  }

  pub fn try_dequeue(&self) -> DequeueResult<Envelope<Msg>> {
    let inner = self.inner.lock().unwrap();
    inner.rx.try_recv().map_err(|_| QueueEmpty)
  }

  pub fn non_empty(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    match inner.rx.try_recv() {
      Ok(_) => true,
      Err(_) => false,
    }
  }

  pub fn is_empty(&self) -> bool {
    !self.non_empty()
  }
}
