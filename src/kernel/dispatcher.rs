use crate::kernel::queue::{EnqueueResult, QueueWriter};
use crate::kernel::{Envelope, Message};

#[derive(Clone)]
pub struct Dispatcher<M: Message> {
  queue: QueueWriter<M>,
}

impl<M: Message> Dispatcher<M> {
  pub fn new(queue: QueueWriter<M>) -> Self {
    Self { queue }
  }

  pub fn try_enqueue(&self, msg: Envelope<M>) -> EnqueueResult<M> {
    self.queue.try_enqueue(msg)
  }
}

unsafe impl<M: Message> Send for Dispatcher<M> {}

unsafe impl<M: Message> Sync for Dispatcher<M> {}
