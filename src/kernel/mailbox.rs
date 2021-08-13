use std::sync::{Arc, Mutex};

use anyhow::Result;

use num_enum::TryFromPrimitive;
use std::convert::TryFrom;

use crate::kernel::{Envelope, Message};
use crate::kernel::queue::*;
use crate::actor::ExtendedCell;

#[derive(Debug, Clone, PartialEq, Eq, TryFromPrimitive)]
#[repr(u32)]
pub enum MailboxStatus {
  Open = 0,
  Closed = 1,
  Scheduled = 2,
  ShouldScheduleMask = 3,
  ShouldNotProcessMask = !2,
  SuspendMask = !3,
  SuspendUnit = 6,
}

#[derive(Clone)]
pub struct Mailbox<M: Message> {
  inner: Arc<Mutex<MailboxInner<M>>>,
}

struct MailboxInner<M: Message> {
  queue: Arc<dyn QueueReader<M>>,
  actor: Option<ExtendedCell<M>>,
  current_status: u32,
  limit: u32,
  count: u32,
}

impl<M: Message> Mailbox<M> {
  pub fn new(limit: u32, queue: impl QueueReader<M> + 'static) -> Self {
    Self {
      inner: Arc::from(Mutex::new(MailboxInner {
        queue: Arc::from(queue),
        actor: None,
        current_status: MailboxStatus::Open as u32,
        limit,
        count: 0,
      })),
    }
  }

  pub fn set_actor(&self, cell: ExtendedCell<M>) {
    let mut inner = self.inner.lock().unwrap();
    inner.actor = Some(cell)
  }

  pub fn should_process_message(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    (inner.current_status & MailboxStatus::ShouldNotProcessMask as u32) == 0
  }

  pub fn suspend_count(&self) -> u32 {
    let inner = self.inner.lock().unwrap();
    inner.current_status / MailboxStatus::SuspendMask as u32
  }

  pub fn is_suspend(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    (inner.current_status & MailboxStatus::SuspendMask as u32) != 0
  }

  pub fn is_closed(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    let current_status = MailboxStatus::try_from(inner.current_status).unwrap();
    current_status == MailboxStatus::Closed
  }

  pub fn is_scheduled(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    (inner.current_status & MailboxStatus::Scheduled as u32) != 0
  }

  pub fn update_status(&self, old: u32, new: u32) -> bool {
    let mut inner = self.inner.lock().unwrap();
    if inner.current_status == old {
      inner.current_status = new;
      true
    } else {
      false
    }
  }

  pub fn set_status(&self, value: u32) {
    let mut inner = self.inner.lock().unwrap();
    inner.current_status = value;
  }

  pub fn resume(&self) -> bool {
    loop {
      let inner = self.inner.lock().unwrap();
      let current_status = MailboxStatus::try_from(inner.current_status).unwrap();
      if current_status == MailboxStatus::Closed {
        self.set_status(MailboxStatus::Closed as u32);
        return false;
      }
      let s = inner.current_status;
      let next = if s < MailboxStatus::SuspendUnit as u32 {
        s
      } else {
        s - MailboxStatus::SuspendUnit as u32
      };
      if self.update_status(inner.current_status, next) {
        return next < MailboxStatus::SuspendUnit as u32;
      }
    }
  }

  pub fn suspend(&self) -> bool {
    loop {
      let inner = self.inner.lock().unwrap();
      let current_status = MailboxStatus::try_from(inner.current_status).unwrap();
      if current_status == MailboxStatus::Closed {
        self.set_status(MailboxStatus::Closed as u32);
        return false;
      }
      let s = inner.current_status;
      if self.update_status(s, s + MailboxStatus::SuspendUnit as u32) {
        return s < MailboxStatus::SuspendUnit as u32;
      }
    }
  }

  pub fn become_closed(&self) -> bool {
    loop {
      let inner = self.inner.lock().unwrap();
      let current_status = MailboxStatus::try_from(inner.current_status).unwrap();
      if current_status == MailboxStatus::Closed {
        self.set_status(MailboxStatus::Closed as u32);
        return false;
      }
      let s = inner.current_status;
      if self.update_status(s, MailboxStatus::Closed as u32) {
        return true;
      }
    }
  }

  pub fn set_as_scheduled(&self) -> bool {
    loop {
      let inner = self.inner.lock().unwrap();
      let s = inner.current_status;
      if (s & MailboxStatus::ShouldScheduleMask as u32) != MailboxStatus::Open as u32 {
        return false;
      }
      if self.update_status(s, s | MailboxStatus::Scheduled as u32) {
        return true;
      }
    }
  }

  pub fn set_as_idle(&self) -> bool {
    loop {
      let inner = self.inner.lock().unwrap();
      let s = inner.current_status;
      if self.update_status(s, s & !(MailboxStatus::Scheduled as u32)) {
        return true;
      }
    }
  }

  fn process_mailbox(&mut self, _left: u32, _dead_line_ns: u64) {
    if self.should_process_message() {
      let next = self.dequeue_opt();
      if next.is_some() {}
    }
  }

  pub fn reset_dequeue_count(&mut self) {
    let mut inner = self.inner.lock().unwrap();
    inner.count = 0
  }

  pub fn dequeue(&mut self) -> Envelope<M> {
    let inner = self.inner.lock().unwrap();
    inner.queue.dequeue()
  }

  pub fn dequeue_opt(&mut self) -> Option<Envelope<M>> {
    let inner = self.inner.lock().unwrap();
    inner.queue.dequeue_opt()
  }

  pub fn try_dequeue(&mut self) -> Result<Option<Envelope<M>>> {
    let inner = self.inner.lock().unwrap();
    inner.queue.try_dequeue()
  }

  pub fn dequeue_aware_limit(&mut self) -> Option<Envelope<M>> {
    let mut inner = self.inner.lock().unwrap();
    if inner.count < inner.limit {
      inner.count += 1;
      Some(inner.queue.dequeue())
    } else {
      None
    }
  }

  pub fn try_dequeue_aware_limit(&mut self) -> Option<Result<Option<Envelope<M>>>> {
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

  pub fn number_of_messages(&self) -> usize {
    let inner = self.inner.lock().unwrap();
    inner.queue.number_of_messages()
  }
}

#[derive(Clone)]
pub struct MailboxSender<M: Message> {
  queue: Arc<dyn QueueWriter<M>>
}

impl<M: Message> MailboxSender<M> {
  pub fn new(queue: impl QueueWriter<M> + 'static) -> Self {
    Self { queue: Arc::from(queue) }
  }

  pub fn try_enqueue(&self, _cell: ExtendedCell<M>, msg: Envelope<M>) -> Result<()> {
    self.queue.try_enqueue(msg)
  }
}
//
// unsafe impl<M: Message> Send for MailboxSender<M> {}
//
// unsafe impl<M: Message> Sync for MailboxSender<M> {}
