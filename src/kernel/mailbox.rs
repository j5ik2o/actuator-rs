use std::ops::Deref;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::kernel::{Envelope, Message};
use crate::kernel::dispatcher::Dispatcher;
use crate::kernel::queue::*;

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
  inner: Arc<Mutex<MailboxInner<M>>>,
}

struct MailboxInner<M: Message> {
  queue: QueueReader<M>,
  limit: u32,
  count: u32,
}

impl<M: Message> Mailbox<M> {
  pub fn new(limit: u32, queue: QueueReader<M>) -> Self {
    Self {
      inner: Arc::from(Mutex::new(MailboxInner {
        queue,
        limit,
        count: 0,
      })),
    }
  }

  pub fn reset_dequeue_count(&mut self) {
    let mut inner = self.inner.lock().unwrap();
    inner.count = 0
  }

  pub fn dequeue(&mut self) -> Option<Envelope<M>> {
    let mut inner = self.inner.lock().unwrap();
    if inner.count < inner.limit {
      inner.count += 1;
      Some(inner.queue.dequeue())
    } else {
      None
    }
  }

  pub fn try_dequeue(&mut self) -> Option<Result<Option<Envelope<M>>>> {
    let mut inner = self.inner.lock().unwrap();
    if inner.count < inner.limit {
      inner.count += 1;
      Some(inner.queue.try_dequeue())
    } else {
      None
    }
  }

  pub fn non_empty(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    inner.queue.non_empty()
  }

  pub fn is_empty(&self) -> bool {
    !self.non_empty()
  }
}
