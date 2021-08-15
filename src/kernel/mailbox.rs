use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::Result;

use crate::actor::ExtendedCell;
use crate::kernel::{Envelope, Message};
use crate::kernel::mailbox_status::MailboxStatus;
use crate::kernel::queue::*;

#[derive(Clone)]
pub struct MailboxSender<M: Message> {
  queue: Arc<dyn QueueWriter<M>>,
}

impl<M: Message> MailboxSender<M> {
  pub fn new(queue: impl QueueWriter<M> + 'static) -> Self {
    Self {
      queue: Arc::from(queue),
    }
  }
  pub fn from_arc(queue: Arc<dyn QueueWriter<M>>) -> Self {
    Self { queue }
  }

  pub fn try_enqueue(&self, _cell: ExtendedCell<M>, msg: Envelope<M>) -> Result<()> {
    self.queue.try_enqueue(msg)
  }
}

unsafe impl<M: Message> Send for MailboxSender<M> {}

unsafe impl<M: Message> Sync for MailboxSender<M> {}

#[derive(Clone)]
pub struct Mailbox<M: Message> {
  inner: Arc<Mutex<MailboxInner<M>>>,
}

struct MailboxInner<M: Message> {
  queue_reader: Arc<dyn QueueReader<M>>,
  queue_writer: Arc<dyn QueueWriter<M>>,
  actor: Option<ExtendedCell<M>>,
  current_status: u32,
  limit: u32,
  count: u32,
}

impl<M: Message> Mailbox<M> {
  pub fn new(
    limit: u32,
    queue_reader: impl QueueReader<M> + 'static,
    queue_writer: impl QueueWriter<M> + 'static,
  ) -> Self {
    Self {
      inner: Arc::from(Mutex::new(MailboxInner {
        queue_reader: Arc::from(queue_reader),
        queue_writer: Arc::from(queue_writer),
        actor: None,
        current_status: MailboxStatus::Open as u32,
        limit,
        count: 0,
      })),
    }
  }

  pub fn new_sender(&self) -> MailboxSender<M> {
    let inner = self.inner.lock().unwrap();
    let new_queue_writer = Arc::clone(&inner.queue_writer);
    MailboxSender::from_arc(new_queue_writer)
  }

  pub fn set_actor(&mut self, cell: ExtendedCell<M>) {
    let mut inner = self.inner.lock().unwrap();
    inner.actor = Some(cell)
  }

  pub fn should_process_message(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    (inner.current_status & MailboxStatus::ShouldNotProcessMask as u32) == 0
  }

  pub fn suspend_count(&self) -> u32 {
    let inner = self.inner.lock().unwrap();
    debug!(
      "current_status = {}, suspend_mask = {}",
      inner.current_status,
      MailboxStatus::SuspendMask as u32
    );
    inner.current_status / MailboxStatus::SuspendUnit as u32
  }

  pub fn is_suspend(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    (inner.current_status & MailboxStatus::SuspendMask as u32) != 0
  }

  pub fn is_closed(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    inner.current_status == MailboxStatus::Closed as u32
  }

  pub fn is_scheduled(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    (inner.current_status & MailboxStatus::Scheduled as u32) != 0
  }

  fn _update_status(inner: &mut MutexGuard<MailboxInner<M>>, old: u32, new: u32) -> bool {
    debug!(
      "current_status = {}, old = {}, new = {}",
      inner.current_status, old, new
    );
    if inner.current_status == old {
      inner.current_status = new;
      true
    } else {
      false
    }
  }

  fn _set_status(inner: &mut MutexGuard<MailboxInner<M>>, value: u32) {
    inner.current_status = value;
  }

  pub fn resume(&self) -> bool {
    loop {
      let mut inner = self.inner.lock().unwrap();
      if inner.current_status == MailboxStatus::Closed as u32 {
        Self::_set_status(&mut inner, MailboxStatus::Closed as u32);
        return false;
      }
      let s = inner.current_status;
      let next = if s < MailboxStatus::SuspendUnit as u32 {
        s
      } else {
        s - MailboxStatus::SuspendUnit as u32
      };
      if Self::_update_status(&mut inner, s, next) {
        return next < MailboxStatus::SuspendUnit as u32;
      }
    }
  }

  pub fn suspend(&self) -> bool {
    loop {
      let mut inner = self.inner.lock().unwrap();
      if inner.current_status == MailboxStatus::Closed as u32 {
        Self::_set_status(&mut inner, MailboxStatus::Closed as u32);
        debug!("Closed: suspend: false");
        return false;
      }
      let s = inner.current_status;
      if Self::_update_status(&mut inner, s, s + MailboxStatus::SuspendUnit as u32) {
        let result = s < MailboxStatus::SuspendUnit as u32;
        debug!(
          "suspend: s = {}, suspend_unit = {}, result = {}",
          s,
          MailboxStatus::SuspendUnit as u32,
          result
        );
        return result;
      }
    }
  }

  pub fn become_closed(&self) -> bool {
    loop {
      let mut inner = self.inner.lock().unwrap();
      if inner.current_status == MailboxStatus::Closed as u32 {
        Self::_set_status(&mut inner, MailboxStatus::Closed as u32);
        debug!("become_closed: false");
        return false;
      }
      let s = inner.current_status;
      if Self::_update_status(&mut inner, s, MailboxStatus::Closed as u32) {
        debug!("become_closed: true");
        return true;
      }
    }
  }

  pub fn set_as_scheduled(&mut self) -> bool {
    loop {
      let mut inner = self.inner.lock().unwrap();
      let s = inner.current_status;
      if (s & MailboxStatus::ShouldScheduleMask as u32) != MailboxStatus::Open as u32 {
        debug!("set_as_scheduled: false");
        return false;
      }
      if Self::_update_status(&mut inner, s, s | MailboxStatus::Scheduled as u32) {
        debug!("set_as_scheduled: true");
        return true;
      }
    }
  }

  pub fn set_as_idle(&mut self) -> bool {
    loop {
      let mut inner = self.inner.lock().unwrap();
      let s = inner.current_status;
      if Self::_update_status(&mut inner, s, s & !(MailboxStatus::Scheduled as u32)) {
        return true;
      }
    }
  }

  pub fn can_be_scheduled_for_execution(
    &self,
    has_message_hint: bool,
    has_system_message_hint: bool,
  ) -> bool {
    let inner = self.inner.lock().unwrap();
    match inner.current_status {
      cs if cs == MailboxStatus::Open as u32 || cs == MailboxStatus::Scheduled as u32 => {
        has_message_hint || has_system_message_hint || self.has_messages()
      }
      cs if cs == MailboxStatus::Closed as u32 => false,
      _ => has_system_message_hint || self.has_system_messages(),
    }
  }

  // TODO:
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

  pub fn try_enqueue(&self, _cell: ExtendedCell<M>, msg: Envelope<M>) -> Result<()> {
    let inner = self.inner.lock().unwrap();
    inner.queue_writer.try_enqueue(msg)
  }

  pub fn dequeue(&mut self) -> Envelope<M> {
    let inner = self.inner.lock().unwrap();
    inner.queue_reader.dequeue()
  }

  pub fn dequeue_opt(&mut self) -> Option<Envelope<M>> {
    let inner = self.inner.lock().unwrap();
    inner.queue_reader.dequeue_opt()
  }

  pub fn try_dequeue(&mut self) -> Result<Option<Envelope<M>>> {
    let inner = self.inner.lock().unwrap();
    inner.queue_reader.try_dequeue()
  }

  pub fn dequeue_aware_limit(&mut self) -> Option<Envelope<M>> {
    let mut inner = self.inner.lock().unwrap();
    if inner.count < inner.limit {
      inner.count += 1;
      Some(inner.queue_reader.dequeue())
    } else {
      None
    }
  }

  pub fn try_dequeue_aware_limit(&mut self) -> Option<Result<Option<Envelope<M>>>> {
    let mut inner = self.inner.lock().unwrap();
    if inner.count < inner.limit {
      inner.count += 1;
      Some(inner.queue_reader.try_dequeue())
    } else {
      None
    }
  }

  pub fn has_system_messages(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    inner.queue_reader.non_empty()
  }

  pub fn has_messages(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    inner.queue_reader.non_empty()
  }

  pub fn number_of_messages(&self) -> usize {
    let inner = self.inner.lock().unwrap();
    inner.queue_reader.number_of_messages()
  }
}
