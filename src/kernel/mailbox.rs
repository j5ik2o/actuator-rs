use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Instant, SystemTime, Duration};

use anyhow::Result;
use async_trait::async_trait;
use num_enum::TryFromPrimitive;

use crate::kernel::{ActorCell, ActorRef, Envelope};
use crate::kernel::mailbox::queue::{EnvelopeQueue, MessageSize};
use std::cmp::max;
use crate::kernel::system_message::SystemMessage;

mod queue;

#[derive(Debug, Clone, PartialEq, Eq, TryFromPrimitive)]
#[repr(u32)]
pub enum MailboxStatus {
  Open = 0,
  Closed = 1,
  Scheduled = 2,
  ShouldScheduleMask = 3,
  ShouldNotProcessMask = !2,
  SuspendMask = !3,
  SuspendUnit = 4,
}

pub trait SystemMessageQueue {
  fn system_enqueue(&mut self, receiver: &dyn ActorRef, message: SystemMessage);
}

#[async_trait]
pub trait Mailbox {
  fn enqueue(&mut self, receiver: &dyn ActorRef, msg: Envelope) -> Result<()>;
  fn dequeue(&mut self) -> Result<Envelope>;

  fn has_messages(&self) -> bool;
  fn has_system_messages(&self) -> bool;
  fn member_of_messages(&self) -> MessageSize;

  fn should_process_message(&self) -> bool;
  fn suspend_count(&self) -> u32;

  fn is_suspend(&self) -> bool;
  fn is_closed(&self) -> bool;
  fn is_scheduled(&self) -> bool;

  fn resume(&self) -> bool;
  fn suspend(&self) -> bool;
  fn become_closed(&self) -> bool;

  fn set_as_scheduled(&mut self) -> bool;
  fn set_as_idle(&mut self) -> bool;
  fn can_be_scheduled_for_execution(
    &self,
    has_message_hint: bool,
    has_system_message_hint: bool,
  ) -> bool;

  fn process_all_system_messages(&mut self);

  fn run(&mut self);
}

pub struct DefaultMailbox {
  inner: Arc<Mutex<DefaultMailboxInner>>,
}

struct DefaultMailboxInner {
  queue: Arc<Mutex<dyn EnvelopeQueue>>,
  actor: Option<Arc<Mutex<dyn ActorCell>>>,
  current_status: AtomicU32,
  throughput: usize,
  is_throughput_deadline_time_defined: bool,
  throughput_deadline_time: Duration,
}

impl DefaultMailbox {
  pub fn new(queue: Arc<Mutex<dyn EnvelopeQueue>>) -> Self {
    Self {
      inner: Arc::new(Mutex::new(DefaultMailboxInner {
        queue,
        actor: None,
        current_status: AtomicU32::new(0),
        throughput: 1,
        is_throughput_deadline_time_defined: false,
        throughput_deadline_time: Duration::from_secs(1),
      })),
    }
  }

  fn process_mailbox(&mut self) {
    let (left, deadline_ns) = {
      let inner = self.inner.lock().unwrap();
      let l = max(inner.throughput, 1);
      let d = if inner.is_throughput_deadline_time_defined {
        let now = SystemTime::now();
        now.elapsed().unwrap().as_nanos() + inner.throughput_deadline_time.as_nanos()
      } else {
        0
      };
      (l, d)
    };
    self.process_mailbox_with(left, deadline_ns)
  }

  fn is_throughput_deadline_time_defined(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    let is_throughput_deadline_time_defined = inner.is_throughput_deadline_time_defined;
    is_throughput_deadline_time_defined
  }

  fn process_mailbox_with(&mut self, left: usize, deadline_ns: u128) {
    if self.should_process_message() {
      match self.dequeue() {
        Ok(next) => {
          let is_throughput_deadline_time_defined = self.is_throughput_deadline_time_defined();
          {
            let inner = self.inner.lock().unwrap();
            let mut actor = inner.actor.as_ref().unwrap().lock().unwrap();
            actor.invoke(&next);
          }
          self.process_all_system_messages();
          let now = SystemTime::now();
          if left > 1
            && (!is_throughput_deadline_time_defined
              || (now.elapsed().unwrap().as_nanos() as u128 - deadline_ns) < 0)
          {
            self.process_mailbox_with(left - 1, deadline_ns)
          }
        }
        Err(err) => {}
      }
    }
  }

  fn update_status(inner: &mut MutexGuard<DefaultMailboxInner>, old: u32, new: u32) -> bool {
    let current_status = inner.current_status.load(Ordering::Relaxed);
    log::debug!(
      "current_status = {}, old = {}, new = {}",
      current_status,
      old,
      new
    );
    if current_status == old {
      inner.current_status.store(new, Ordering::Relaxed);
      true
    } else {
      false
    }
  }

  fn set_status(inner: &mut MutexGuard<DefaultMailboxInner>, value: u32) {
    inner.current_status.store(value, Ordering::Relaxed);
  }
}

#[async_trait]
impl Mailbox for DefaultMailbox {
  fn enqueue(&mut self, receiver: &dyn ActorRef, msg: Envelope) -> Result<()> {
    let inner = self.inner.lock().unwrap();
    let mut queue = inner.queue.lock().unwrap();
    queue.enqueue_with_receiver(receiver, msg)
  }

  fn dequeue(&mut self) -> Result<Envelope> {
    let inner = self.inner.lock().unwrap();
    let mut queue = inner.queue.lock().unwrap();
    queue.dequeue()
  }

  fn has_messages(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    let queue = inner.queue.lock().unwrap();
    queue.has_messages()
  }

  fn has_system_messages(&self) -> bool {
    todo!()
  }

  fn member_of_messages(&self) -> MessageSize {
    let inner = self.inner.lock().unwrap();
    let queue = inner.queue.lock().unwrap();
    queue.number_of_messages()
  }

  fn should_process_message(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    let current_status = inner.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::ShouldNotProcessMask as u32) == 0
  }

  fn suspend_count(&self) -> u32 {
    let inner = self.inner.lock().unwrap();
    let current_status = inner.current_status.load(Ordering::Relaxed);
    log::debug!(
      "current_status = {}, suspend_mask = {}",
      current_status,
      MailboxStatus::SuspendMask as u32
    );
    current_status / MailboxStatus::SuspendUnit as u32
  }

  fn is_suspend(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    let current_status = inner.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::SuspendMask as u32) != 0
  }

  fn is_closed(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    let current_status = inner.current_status.load(Ordering::Relaxed);
    current_status == MailboxStatus::Closed as u32
  }

  fn is_scheduled(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    let current_status = inner.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::Scheduled as u32) != 0
  }

  fn resume(&self) -> bool {
    loop {
      let mut inner = self.inner.lock().unwrap();
      let current_status = inner.current_status.load(Ordering::Relaxed);
      if current_status == MailboxStatus::Closed as u32 {
        Self::set_status(&mut inner, MailboxStatus::Closed as u32);
        return false;
      }
      let s = current_status;
      let next = if s < MailboxStatus::SuspendUnit as u32 {
        s
      } else {
        s - MailboxStatus::SuspendUnit as u32
      };
      if Self::update_status(&mut inner, s, next) {
        return next < MailboxStatus::SuspendUnit as u32;
      }
    }
  }

  fn suspend(&self) -> bool {
    loop {
      let mut inner = self.inner.lock().unwrap();
      let current_status = inner.current_status.load(Ordering::Relaxed);
      if current_status == MailboxStatus::Closed as u32 {
        Self::set_status(&mut inner, MailboxStatus::Closed as u32);
        log::debug!("Closed: suspend: false");
        return false;
      }
      let s = current_status;
      if Self::update_status(&mut inner, s, s + MailboxStatus::SuspendUnit as u32) {
        let result = s < MailboxStatus::SuspendUnit as u32;
        log::debug!(
          "suspend: s = {}, suspend_unit = {}, result = {}",
          s,
          MailboxStatus::SuspendUnit as u32,
          result
        );
        return result;
      }
    }
  }

  fn become_closed(&self) -> bool {
    loop {
      let mut inner = self.inner.lock().unwrap();
      let current_status = inner.current_status.load(Ordering::Relaxed);
      if current_status == MailboxStatus::Closed as u32 {
        Self::set_status(&mut inner, MailboxStatus::Closed as u32);
        log::debug!("become_closed: false");
        return false;
      }
      let s = current_status;
      if Self::update_status(&mut inner, s, MailboxStatus::Closed as u32) {
        log::debug!("become_closed: true");
        return true;
      }
    }
  }

  fn set_as_scheduled(&mut self) -> bool {
    loop {
      let mut inner = self.inner.lock().unwrap();
      let current_status = inner.current_status.load(Ordering::Relaxed);
      let s = current_status;
      if (s & MailboxStatus::ShouldScheduleMask as u32) != MailboxStatus::Open as u32 {
        log::debug!("set_as_scheduled: false");
        return false;
      }
      if Self::update_status(&mut inner, s, s | MailboxStatus::Scheduled as u32) {
        log::debug!("set_as_scheduled: true");
        return true;
      }
    }
  }

  fn set_as_idle(&mut self) -> bool {
    loop {
      let mut inner = self.inner.lock().unwrap();
      let current_status = inner.current_status.load(Ordering::Relaxed);
      let s = current_status;
      if Self::update_status(&mut inner, s, s & !(MailboxStatus::Scheduled as u32)) {
        return true;
      }
    }
  }

  fn can_be_scheduled_for_execution(
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
      _ => has_system_message_hint || self.has_system_messages(),
    }
  }

  fn process_all_system_messages(&mut self) {
    todo!()
  }

  fn run(&mut self) {
    if !self.is_closed() {
      self.process_all_system_messages();
      self.process_mailbox();
    }
    self.set_as_idle();
    // dispatcher.registerForExecution(this, false, false)
  }
}
