use std::fmt::{Debug};
use std::sync::mpsc::Sender;

use anyhow::Result;
use thiserror::Error;

use crate::kernel::{Envelope, Message};

pub trait QueueWriter<M: Message> {
  fn try_enqueue(&self, msg: Envelope<M>) -> Result<()>;
}

#[derive(Clone)]
pub struct QueueWriterInMPSC<M: Message> {
  tx: Sender<Envelope<M>>,
}

#[derive(Clone, Debug, Error)]
pub enum EnqueueError<A: Debug> {
  #[error("send error: {0:?}")]
  SendError(A),
}

impl<M: Message> QueueWriterInMPSC<M> {
  pub fn new(tx: Sender<Envelope<M>>) -> Self {
    Self { tx }
  }
}

impl<M: Message> QueueWriter<M> for QueueWriterInMPSC<M> {

  fn try_enqueue(&self, msg: Envelope<M>) -> Result<()> {
    match self.tx.send(msg) {
      Ok(_) => Ok(()),
      Err(e) => Err(EnqueueError::SendError(e.0))?,
    }
  }
}
