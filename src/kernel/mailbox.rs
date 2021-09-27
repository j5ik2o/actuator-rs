use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, Duration};

use anyhow::Result;
use async_trait::async_trait;
use num_enum::TryFromPrimitive;

use crate::kernel::{ActorCell, ActorRef, Envelope};
use crate::kernel::mailbox::queue::{EnvelopeQueue, MessageSize};
use std::cmp::max;
use crate::kernel::system_message::{
  SystemMessage, LatestFirstSystemMessageList, EarliestFirstSystemMessageList, SystemMessageList,
};
use crate::kernel::system_message::SystemMessage::NoMessage;

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
  fn system_drain(
    &mut self,
    new_contents: &LatestFirstSystemMessageList,
  ) -> EarliestFirstSystemMessageList;
  fn has_system_messages(&self) -> bool;
}

#[async_trait]
pub trait Mailbox {
  fn enqueue(&mut self, receiver: &dyn ActorRef, msg: Envelope) -> Result<()>;
  fn dequeue(&mut self) -> Result<Envelope>;

  fn has_messages(&self) -> bool;
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
  system_message: Option<SystemMessage>,
  dead_letter_mailbox: Option<Arc<Mutex<dyn SystemMessageQueue>>>,
}

impl SystemMessageQueue for DefaultMailbox {
  fn system_enqueue(&mut self, receiver: &dyn ActorRef, mut message: SystemMessage) {
    let current_list = self.system_queue_get();
    let head_opt = current_list.head();
    if head_opt.iter().any(|head| {
      let v_inner = head.lock().unwrap();
      v_inner.is_no_message()
    }) {
      let mailbox_inner = self.inner.lock().unwrap();
      match &mailbox_inner.dead_letter_mailbox {
        Some(system_message_queue_arc) => {
          log::debug!("send message to dead letter mailbox");
          let mut system_message_queue = system_message_queue_arc.lock().unwrap();
          system_message_queue.system_enqueue(receiver, message.clone());
        }
        None => {
          log::warn!("The message was not delivered to the dead letter.");
        }
      }
    } else {
      if !self.system_queue_put(
        &current_list.clone(),
        &current_list.prepend(message.clone()),
      ) {
        message.unlink();
        self.system_enqueue(receiver, message);
      }
    }
  }

  fn system_drain(
    &mut self,
    new_contents: &LatestFirstSystemMessageList,
  ) -> EarliestFirstSystemMessageList {
    let current_list = self.system_queue_get();
    let head_opt = current_list.head();
    if head_opt.iter().any(|head| {
      let v_inner = head.lock().unwrap();
      v_inner.is_no_message()
    }) {
      EarliestFirstSystemMessageList::new(None)
    } else if self.system_queue_put(&current_list, new_contents) {
      current_list.reverse()
    } else {
      self.system_drain(new_contents)
    }
  }

  fn has_system_messages(&self) -> bool {
    let current_list = self.system_queue_get();
    let head_opt = current_list.head();
    head_opt
      .map(|head| {
        let inner = head.lock().unwrap();
        match &*inner {
          NoMessage { .. } => false,
          _ => true,
        }
      })
      .unwrap_or(false)
  }
}

#[cfg(test)]
fn init_logger() {
  use std::env;
  env::set_var("RUST_LOG", "debug");
  // env::set_var("RUST_LOG", "trace");
  let _ = logger::try_init();
}

#[cfg(test)]
mod test_system_message_queue {
  use super::*;
  use crate::kernel::mailbox::queue::VecQueue;
  use crate::kernel::DummyActorRef;
  use crate::kernel::system_message::LNIL;

  #[test]
  fn system_enqueue() {
    init_logger();
    let dead_letter_queue = Arc::new(Mutex::new(VecQueue::<Envelope>::new()));
    let dead_letter_mailbox = DefaultMailbox::of_dead_letter(dead_letter_queue);
    let queue = Arc::new(Mutex::new(VecQueue::<Envelope>::new()));

    let mut mailbox = DefaultMailbox::new(queue, Some(Arc::new(Mutex::new(dead_letter_mailbox))));

    let dmmy_actor_ref = DummyActorRef;
    let system_message = SystemMessage::of_create(None);
    mailbox.system_enqueue(&dmmy_actor_ref, system_message);
    let system_message = SystemMessage::of_resume(None);
    mailbox.system_enqueue(&dmmy_actor_ref, system_message);

    assert!(mailbox.has_system_messages());

    let l = mailbox.system_drain(&LNIL);
    println!("{:?}", l);
    println!("{:?}", l.head());
  }

  #[test]
  fn has_system_messages() {
    init_logger();
    let vec_queue = VecQueue::<Envelope>::new();
    let queue = Arc::new(Mutex::new(vec_queue));
    let dead_letter_mailbox = DefaultMailbox::of_dead_letter(queue);
    assert_eq!(dead_letter_mailbox.has_system_messages(), false);
  }
}

impl DefaultMailbox {
  pub fn new(
    queue: Arc<Mutex<dyn EnvelopeQueue>>,
    dead_letter_mailbox: Option<Arc<Mutex<dyn SystemMessageQueue>>>,
  ) -> Self {
    Self {
      inner: Arc::new(Mutex::new(DefaultMailboxInner {
        queue,
        actor: None,
        current_status: AtomicU32::new(0),
        throughput: 1,
        is_throughput_deadline_time_defined: false,
        throughput_deadline_time: Duration::from_secs(1),
        system_message: None,
        dead_letter_mailbox,
      })),
    }
  }

  pub fn of_dead_letter(queue: Arc<Mutex<dyn EnvelopeQueue>>) -> Self {
    Self::new(queue, None)
  }

  fn system_queue_get(&self) -> LatestFirstSystemMessageList {
    let inner = self.inner.lock().unwrap();
    LatestFirstSystemMessageList::new(inner.system_message.clone())
  }

  fn system_queue_put(
    &mut self,
    old: &LatestFirstSystemMessageList,
    new: &LatestFirstSystemMessageList,
  ) -> bool {
    log::debug!("system_queue_put: old = {:?}, new = {:?}", old, new);
    let same = match (old.head(), new.head()) {
      (Some(v1), Some(v2)) => {
        if (v1.as_ref() as *const _) == (v2.as_ref() as *const _) {
          true
        } else {
          let v1_inner = v1.lock().unwrap();
          let v2_inner = v2.lock().unwrap();
          &*v1_inner == &*v2_inner
        }
      }
      (None, None) => true,
      _ => false,
    };
    if same {
      return true;
    }
    log::debug!("system_queue_put: same = {}", same);
    let mut inner = self.inner.lock().unwrap();
    let result = match new.head() {
      Some(arc) => {
        let new_inner = arc.lock().unwrap();
        if !inner.system_message.contains(&*new_inner) {
          inner.system_message = Some(new_inner.clone());
          true
        } else {
          false
        }
      }
      None => {
        inner.system_message = None;
        true
      }
    };

    log::debug!("system_queue_put: new_old = {:?}, new_new = {:?}", old, new);
    result
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
        Err(_err) => {}
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
