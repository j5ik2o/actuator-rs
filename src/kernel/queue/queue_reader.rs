use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, TryRecvError};

use crate::kernel::{Envelope, Message};

pub struct QueueReader<M: Message> {
  inner: Mutex<QueueReaderInner<M>>,
}

struct QueueReaderInner<M: Message> {
  rx: Receiver<Envelope<M>>,
}
pub enum DequeueError {
  Disconnected,
}
pub type DequeueResult<M> = Result<Option<M>, DequeueError>;

impl<M: Message> QueueReader<M> {
  pub fn new(rx: Receiver<Envelope<M>>) -> Self {
    Self {
      inner: Mutex::new(QueueReaderInner { rx }),
    }
  }

  pub fn dequeue(&self) -> Envelope<M> {
    let inner = self.inner.lock().unwrap();
    inner.rx.recv().unwrap()
  }

  pub fn try_dequeue(&self) -> DequeueResult<Envelope<M>> {
    let inner = self.inner.lock().unwrap();
    match inner.rx.try_recv() {
      Ok(e) => Ok(Some(e)),
      Err(TryRecvError::Empty) => Ok(None),
      Err(TryRecvError::Disconnected) => Err(DequeueError::Disconnected),
    }
  }

  pub fn non_empty(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    match inner.rx.try_recv() {
      Ok(_) => true,
      _ => false,
    }
  }

  pub fn is_empty(&self) -> bool {
    !self.non_empty()
  }
}
