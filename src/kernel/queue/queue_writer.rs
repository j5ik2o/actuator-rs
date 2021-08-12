use crate::kernel::{Envelope, Message};
use std::sync::mpsc::Sender;

#[derive(Clone)]
pub struct QueueWriter<M: Message> {
  tx: Sender<Envelope<M>>,
}

#[derive(Clone, Debug)]
pub struct EnqueueError<A> {
  pub msg: A,
}

pub type EnqueueResult<M> = Result<(), EnqueueError<Envelope<M>>>;

impl<M: Message> QueueWriter<M> {
  pub fn new(tx: Sender<Envelope<M>>) -> Self {
    Self { tx }
  }

  pub fn try_enqueue(&self, msg: Envelope<M>) -> EnqueueResult<M> {
    self
      .tx
      .send(msg)
      .map(|_| ())
      .map_err(|e| EnqueueError { msg: e.0 })
  }
}
