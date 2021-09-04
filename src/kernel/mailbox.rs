use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::Result;

use crate::actor::actor_cell::ActorCell;
use crate::actor::actor_ref::ActorRef;
use crate::actor::ExtendedCell;
use crate::kernel::{MailboxType, new_mailbox};
use crate::kernel::envelope::Envelope;
use crate::kernel::mailbox_sender::MailboxSender;
use crate::kernel::mailbox_status::MailboxStatus;
use crate::kernel::message::Message;
use crate::kernel::queue::*;

#[derive(Debug, Clone)]
pub struct Mailbox<M: Message> {
  inner: Arc<Mutex<MailboxInner<M>>>,
}

#[derive(Debug, Clone)]
struct MailboxInner<M: Message> {
  queue_reader: Arc<dyn QueueReader<M>>,
  queue_writer: Arc<dyn QueueWriter<M>>,
  system_queue_reader: Arc<dyn QueueReader<M>>,
  system_queue_writer: Arc<dyn QueueWriter<M>>,
  actor: Option<ExtendedCell<M>>,
  current_status: Arc<AtomicU32>,
  limit: u32,
  count: u32,
}

impl<M: Message> Default for Mailbox<M> {
  fn default() -> Self {
    new_mailbox(MailboxType::MPSC, u32::MAX)
  }
}

impl<M: Message> Mailbox<M> {
  pub fn new(
    limit: u32,
    queue_reader: impl QueueReader<M> + 'static,
    queue_writer: impl QueueWriter<M> + 'static,
    system_queue_reader: impl QueueReader<M> + 'static,
    system_queue_writer: impl QueueWriter<M> + 'static,
  ) -> Self {
    Self {
      inner: Arc::from(Mutex::new(MailboxInner {
        queue_reader: Arc::from(queue_reader),
        queue_writer: Arc::from(queue_writer),
        system_queue_reader: Arc::from(system_queue_reader),
        system_queue_writer: Arc::from(system_queue_writer),
        actor: None,
        current_status: Arc::new(AtomicU32::from(MailboxStatus::Open as u32)),
        limit,
        count: 0,
      })),
    }
  }

  pub fn new_sender(&self) -> MailboxSender<M> {
    let _inner = self.inner.lock().unwrap();
    MailboxSender::new(self.clone())
  }

  pub fn new_system_sender(&self) -> MailboxSender<M> {
    let _inner = self.inner.lock().unwrap();
    MailboxSender::new(self.clone())
  }

  pub fn set_actor(&mut self, cell: ExtendedCell<M>) {
    let mut inner = self.inner.lock().unwrap();
    inner.actor = Some(cell)
  }

  pub fn get_actor(&self) -> Option<ExtendedCell<M>> {
    let inner = self.inner.lock().unwrap();
    inner.actor.clone()
  }

  pub fn should_process_message(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    let current_status = inner.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::ShouldNotProcessMask as u32) == 0
  }

  pub fn suspend_count(&self) -> u32 {
    let inner = self.inner.lock().unwrap();
    let current_status = inner.current_status.load(Ordering::Relaxed);
    debug!(
      "current_status = {}, suspend_mask = {}",
      current_status,
      MailboxStatus::SuspendMask as u32
    );
    current_status / MailboxStatus::SuspendUnit as u32
  }

  pub fn is_suspend(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    let current_status = inner.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::SuspendMask as u32) != 0
  }

  pub fn is_closed(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    let current_status = inner.current_status.load(Ordering::Relaxed);
    current_status == MailboxStatus::Closed as u32
  }

  pub fn is_scheduled(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    let current_status = inner.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::Scheduled as u32) != 0
  }

  fn _update_status(inner: &mut MutexGuard<MailboxInner<M>>, old: u32, new: u32) -> bool {
    let current_status = inner.current_status.load(Ordering::Relaxed);
    debug!(
      "current_status = {}, old = {}, new = {}",
      current_status, old, new
    );
    if current_status == old {
      inner.current_status.store(new, Ordering::Relaxed);
      true
    } else {
      false
    }
  }

  fn _set_status(inner: &mut MutexGuard<MailboxInner<M>>, value: u32) {
    inner.current_status.store(value, Ordering::Relaxed);
  }

  pub fn resume(&self) -> bool {
    loop {
      let mut inner = self.inner.lock().unwrap();
      let current_status = inner.current_status.load(Ordering::Relaxed);
      if current_status == MailboxStatus::Closed as u32 {
        Self::_set_status(&mut inner, MailboxStatus::Closed as u32);
        return false;
      }
      let s = current_status;
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
      let current_status = inner.current_status.load(Ordering::Relaxed);
      if current_status == MailboxStatus::Closed as u32 {
        Self::_set_status(&mut inner, MailboxStatus::Closed as u32);
        debug!("Closed: suspend: false");
        return false;
      }
      let s = current_status;
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
      let current_status = inner.current_status.load(Ordering::Relaxed);
      if current_status == MailboxStatus::Closed as u32 {
        Self::_set_status(&mut inner, MailboxStatus::Closed as u32);
        debug!("become_closed: false");
        return false;
      }
      let s = current_status;
      if Self::_update_status(&mut inner, s, MailboxStatus::Closed as u32) {
        debug!("become_closed: true");
        return true;
      }
    }
  }

  pub fn set_as_scheduled(&mut self) -> bool {
    loop {
      let mut inner = self.inner.lock().unwrap();
      let current_status = inner.current_status.load(Ordering::Relaxed);
      let s = current_status;
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
      let current_status = inner.current_status.load(Ordering::Relaxed);
      let s = current_status;
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
    let current_status = inner.current_status.load(Ordering::Relaxed);
    match current_status {
      cs if cs == MailboxStatus::Open as u32 || cs == MailboxStatus::Scheduled as u32 => {
        has_message_hint || has_system_message_hint || self.has_messages()
      }
      cs if cs == MailboxStatus::Closed as u32 => false,
      _ => has_system_message_hint || self.has_messages_for_system(),
    }
  }

  // TODO:
  fn process_mailbox(&mut self, _left: u32, _dead_line_ns: u64) {
    if self.should_process_message() {
      let next = self.dequeue_opt();
      if next.is_some() {
        let inner = self.inner.lock().unwrap();
        if let Some(actor) = inner.actor.as_ref() {
          actor.invoke(next.unwrap())
        }
      }
    }
  }

  pub fn reset_dequeue_count(&mut self) {
    let mut inner = self.inner.lock().unwrap();
    inner.count = 0
  }

  pub fn try_enqueue(&self, actor_ref: Arc<dyn ActorRef>, msg: Envelope<M>) -> Result<()> {
    let inner = self.inner.lock().unwrap();
    inner.queue_writer.try_enqueue(Some(actor_ref), msg)
  }

  pub fn try_enqueue_for_system(&self, actor_ref: Arc<dyn ActorRef>, msg: Envelope<M>) -> Result<()> {
    let inner = self.inner.lock().unwrap();
    inner.system_queue_writer.try_enqueue(Some(actor_ref), msg)
  }

  pub fn dequeue(&mut self) -> Envelope<M> {
    let inner = self.inner.lock().unwrap();
    inner.queue_reader.dequeue()
  }

  pub fn dequeue_for_system(&mut self) -> Envelope<M> {
    let inner = self.inner.lock().unwrap();
    inner.system_queue_reader.dequeue()
  }

  pub fn dequeue_opt(&mut self) -> Option<Envelope<M>> {
    let inner = self.inner.lock().unwrap();
    inner.queue_reader.dequeue_opt()
  }

  pub fn dequeue_opt_for_system(&mut self) -> Option<Envelope<M>> {
    let inner = self.inner.lock().unwrap();
    inner.system_queue_reader.dequeue_opt()
  }

  pub fn try_dequeue(&mut self) -> Result<Option<Envelope<M>>> {
    let inner = self.inner.lock().unwrap();
    inner.queue_reader.try_dequeue()
  }

  pub fn try_dequeue_for_system(&mut self) -> Result<Option<Envelope<M>>> {
    let inner = self.inner.lock().unwrap();
    inner.system_queue_reader.try_dequeue()
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

  pub fn has_messages(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    inner.queue_reader.non_empty()
  }

  pub fn has_messages_for_system(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    inner.system_queue_reader.non_empty()
  }

  pub fn number_of_messages(&self) -> MessageSize {
    let inner = self.inner.lock().unwrap();
    inner.queue_reader.number_of_messages()
  }

  pub fn number_of_messages_for_system(&self) -> MessageSize {
    let inner = self.inner.lock().unwrap();
    inner.system_queue_reader.number_of_messages()
  }
}
