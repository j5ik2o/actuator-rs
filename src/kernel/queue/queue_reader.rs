use std::sync::{Mutex};
use std::sync::mpsc::{Receiver, TryRecvError};

use anyhow::Result;
use thiserror::Error;

use crate::kernel::{Envelope, Message};

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

pub struct QueueReaderInMPSC<M: Message> {
  inner: Mutex<QueueReaderInner<M>>,
}

struct QueueReaderInner<M: Message> {
  rx: Receiver<Envelope<M>>,
  next_item: Option<Envelope<M>>,
}

#[derive(Debug, Error)]
pub enum DequeueError {
  #[error("disconnected")]
  Disconnected,
}

impl<M: Message> QueueReaderInMPSC<M> {
  pub fn new(rx: Receiver<Envelope<M>>) -> Self {
    Self {
      inner: Mutex::new(QueueReaderInner {
        rx,
        next_item: None,
      }),
    }
  }
}

impl<M: Message> QueueReader<M> for QueueReaderInMPSC<M> {

  fn dequeue(&self) -> Envelope<M> {
    let mut inner = self.inner.lock().unwrap();
    if let Some(item) = inner.next_item.take() {
      item
    } else {
      inner.rx.recv().unwrap()
    }
  }

  fn try_dequeue(&self) -> Result<Option<Envelope<M>>> {
    let mut inner = self.inner.lock().unwrap();
    if let Some(item) = inner.next_item.take() {
      Ok(Some(item))
    } else {
      match inner.rx.try_recv() {
        Ok(e) => Ok(Some(e)),
        Err(TryRecvError::Empty) => Ok(None),
        Err(TryRecvError::Disconnected) => Err(DequeueError::Disconnected)?,
      }
    }
  }

  fn non_empty(&self) -> bool {
    let mut inner = self.inner.lock().unwrap();
    inner.next_item.is_some() || {
      match inner.rx.try_recv() {
        Ok(item) => {
          inner.next_item = Some(item);
          true
        }
        _ => false,
      }
    }
  }

  fn is_empty(&self) -> bool {
    !self.non_empty()
  }

  fn number_of_messages(&self) -> usize {
    0
  }
}
