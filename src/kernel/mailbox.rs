use crate::kernel::dispatcher::Dispatcher;
use crate::kernel::queue::*;
use crate::kernel::{Envelope, Message};
use std::sync::Arc;

pub enum MailboxStatus {
  Open,
  Closed,
  Scheduled,
  ShouldScheduleMask,
  ShouldNotProcessMask,
  SuspendMask,
  SuspendUnit,
}

#[derive(Clone)]
pub struct Mailbox<M: Message> {
  inner: Arc<MailboxInner<M>>,
}

struct MailboxInner<M: Message> {
  limit: u32,
  queue: QueueReader<M>,
}

impl<M: Message> Mailbox<M> {
  pub fn new(limit: u32, queue: QueueReader<M>) -> Self {
    Self { inner: Arc::from(MailboxInner { limit, queue }) }
  }

  pub fn dequeue(&self) -> Envelope<M> {
    self.inner.queue.dequeue()
  }

  pub fn try_dequeue(&self) -> Result<Envelope<M>, QueueEmpty> {
    self.inner.queue.try_dequeue()
  }

  pub fn non_empty(&self) -> bool {
    self.inner.queue.non_empty()
  }

  pub fn is_empty(&self) -> bool {
    !self.non_empty()
  }
}
