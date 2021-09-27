use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicU32, Ordering, AtomicBool};
use std::time::{SystemTime, Duration};

use anyhow::Result;
use async_trait::async_trait;
use num_enum::TryFromPrimitive;

use crate::kernel::{ActorCell, ActorRef, Envelope};
use crate::kernel::mailbox::queue::{EnvelopeQueue, MessageSize};
use std::cmp::max;
use crate::kernel::system_message::{
  SystemMessage, LatestFirstSystemMessageList, EarliestFirstSystemMessageList, SystemMessageList,
  LNIL,
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
  terminate: AtomicBool,
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
        system_message: None,
        terminate: AtomicBool::new(false),
      })),
    }
  }

  pub fn set_actor(&mut self, actor: Option<Arc<Mutex<dyn ActorCell>>>) {
    let mut inner_guard = self.inner.lock().unwrap();
    inner_guard.actor = actor;
  }

  fn system_queue_get(&self) -> LatestFirstSystemMessageList {
    let inner_guard = self.inner.lock().unwrap();
    LatestFirstSystemMessageList::new(inner_guard.system_message.clone())
  }

  fn system_queue_put(
    &mut self,
    old: &LatestFirstSystemMessageList,
    new: &LatestFirstSystemMessageList,
  ) -> bool {
    log::debug!("system_queue_put: old = {:?}, new = {:?}", old, new);
    let same = match (old.head(), new.head()) {
      (Some(v1_arc), Some(v2_arc)) => {
        if (v1_arc.as_ref() as *const _) == (v2_arc.as_ref() as *const _) {
          true
        } else {
          let v1_guard = v1_arc.lock().unwrap();
          let v2_guard = v2_arc.lock().unwrap();
          &*v1_guard == &*v2_guard
        }
      }
      (None, None) => true,
      _ => false,
    };
    if same {
      return true;
    }
    log::debug!("system_queue_put: same = {}", same);
    let mut inner_guard = self.inner.lock().unwrap();
    let result = match new.head() {
      Some(arc) => {
        let system_message_guard = arc.lock().unwrap();
        inner_guard.system_message = Some(system_message_guard.clone());
        true
      }
      None => {
        inner_guard.system_message = None;
        true
      }
    };

    log::debug!("system_queue_put: new_old = {:?}, new_new = {:?}", old, new);
    result
  }

  fn process_mailbox(&mut self) {
    let (left, deadline_ns) = {
      let inner_guard = self.inner.lock().unwrap();
      let l = max(inner_guard.throughput, 1);
      let d = if inner_guard.is_throughput_deadline_time_defined {
        let now = SystemTime::now();
        now.elapsed().unwrap().as_nanos() + inner_guard.throughput_deadline_time.as_nanos()
      } else {
        0
      };
      (l, d)
    };
    self.process_mailbox_with(left, deadline_ns)
  }

  fn is_throughput_deadline_time_defined(&self) -> bool {
    let inner_guard = self.inner.lock().unwrap();
    let is_throughput_deadline_time_defined = inner_guard.is_throughput_deadline_time_defined;
    is_throughput_deadline_time_defined
  }

  fn process_mailbox_with(&mut self, left: usize, deadline_ns: u128) {
    if self.should_process_message() {
      match self.dequeue() {
        Ok(next) => {
          let is_throughput_deadline_time_defined = self.is_throughput_deadline_time_defined();
          {
            let inner_guard = self.inner.lock().unwrap();
            let mut actor_cell_guard = inner_guard.actor.as_ref().unwrap().lock().unwrap();
            actor_cell_guard.invoke(&next);
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

  fn update_status(inner_guard: &mut MutexGuard<DefaultMailboxInner>, old: u32, new: u32) -> bool {
    let current_status = inner_guard.current_status.load(Ordering::Relaxed);
    log::debug!(
      "current_status = {}, old = {}, new = {}",
      current_status,
      old,
      new
    );
    if current_status == old {
      inner_guard.current_status.store(new, Ordering::Relaxed);
      true
    } else {
      false
    }
  }

  fn set_status(inner_guard: &mut MutexGuard<DefaultMailboxInner>, value: u32) {
    inner_guard.current_status.store(value, Ordering::Relaxed);
  }
}

#[async_trait]
impl Mailbox for DefaultMailbox {
  fn enqueue(&mut self, receiver: &dyn ActorRef, msg: Envelope) -> Result<()> {
    let inner_guard = self.inner.lock().unwrap();
    let mut queue_guard = inner_guard.queue.lock().unwrap();
    queue_guard.enqueue_with_receiver(receiver, msg)
  }

  fn dequeue(&mut self) -> Result<Envelope> {
    let inner_guard = self.inner.lock().unwrap();
    let mut queue_guard = inner_guard.queue.lock().unwrap();
    queue_guard.dequeue()
  }

  fn has_messages(&self) -> bool {
    let inner_guard = self.inner.lock().unwrap();
    let queue_guard = inner_guard.queue.lock().unwrap();
    queue_guard.has_messages()
  }

  fn member_of_messages(&self) -> MessageSize {
    let inner_guard = self.inner.lock().unwrap();
    let queue_guard = inner_guard.queue.lock().unwrap();
    queue_guard.number_of_messages()
  }

  fn should_process_message(&self) -> bool {
    let inner_guard = self.inner.lock().unwrap();
    let current_status = inner_guard.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::ShouldNotProcessMask as u32) == 0
  }

  fn suspend_count(&self) -> u32 {
    let inner_guard = self.inner.lock().unwrap();
    let current_status = inner_guard.current_status.load(Ordering::Relaxed);
    log::debug!(
      "current_status = {}, suspend_mask = {}",
      current_status,
      MailboxStatus::SuspendMask as u32
    );
    current_status / MailboxStatus::SuspendUnit as u32
  }

  fn is_suspend(&self) -> bool {
    let inner_guard = self.inner.lock().unwrap();
    let current_status = inner_guard.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::SuspendMask as u32) != 0
  }

  fn is_closed(&self) -> bool {
    let inner_guard = self.inner.lock().unwrap();
    let current_status = inner_guard.current_status.load(Ordering::Relaxed);
    current_status == MailboxStatus::Closed as u32
  }

  fn is_scheduled(&self) -> bool {
    let inner_guard = self.inner.lock().unwrap();
    let current_status = inner_guard.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::Scheduled as u32) != 0
  }

  fn resume(&self) -> bool {
    loop {
      let mut inner_guard = self.inner.lock().unwrap();
      let current_status = inner_guard.current_status.load(Ordering::Relaxed);
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
      if Self::update_status(&mut inner_guard, s, next) {
        return next < MailboxStatus::SuspendUnit as u32;
      }
    }
  }

  fn suspend(&self) -> bool {
    loop {
      let mut inner_guard = self.inner.lock().unwrap();
      let current_status = inner_guard.current_status.load(Ordering::Relaxed);
      if current_status == MailboxStatus::Closed as u32 {
        Self::set_status(&mut inner, MailboxStatus::Closed as u32);
        log::debug!("Closed: suspend: false");
        return false;
      }
      let s = current_status;
      if Self::update_status(&mut inner_guard, s, s + MailboxStatus::SuspendUnit as u32) {
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
      let mut inner_guard = self.inner.lock().unwrap();
      let current_status = inner_guard.current_status.load(Ordering::Relaxed);
      if current_status == MailboxStatus::Closed as u32 {
        Self::set_status(&mut inner, MailboxStatus::Closed as u32);
        log::debug!("become_closed: false");
        return false;
      }
      let s = current_status;
      if Self::update_status(&mut inner_guard, s, MailboxStatus::Closed as u32) {
        log::debug!("become_closed: true");
        return true;
      }
    }
  }

  fn set_as_scheduled(&mut self) -> bool {
    loop {
      let mut inner_guard = self.inner.lock().unwrap();
      let current_status = inner_guard.current_status.load(Ordering::Relaxed);
      let s = current_status;
      if (s & MailboxStatus::ShouldScheduleMask as u32) != MailboxStatus::Open as u32 {
        log::debug!("set_as_scheduled: false");
        return false;
      }
      if Self::update_status(&mut inner_guard, s, s | MailboxStatus::Scheduled as u32) {
        log::debug!("set_as_scheduled: true");
        return true;
      }
    }
  }

  fn set_as_idle(&mut self) -> bool {
    loop {
      let mut inner_guard = self.inner.lock().unwrap();
      let current_status = inner_guard.current_status.load(Ordering::Relaxed);
      let s = current_status;
      if Self::update_status(&mut inner_guard, s, s & !(MailboxStatus::Scheduled as u32)) {
        return true;
      }
    }
  }

  fn can_be_scheduled_for_execution(
    &self,
    has_message_hint: bool,
    has_system_message_hint: bool,
  ) -> bool {
    let inner_guard = self.inner.lock().unwrap();
    let current_status = inner_guard.current_status.load(Ordering::Relaxed);
    match current_status {
      cs if cs == MailboxStatus::Open as u32 || cs == MailboxStatus::Scheduled as u32 => {
        has_message_hint || has_system_message_hint || self.has_messages()
      }
      cs if cs == MailboxStatus::Closed as u32 => false,
      _ => has_system_message_hint || self.has_system_messages(),
    }
  }

  fn process_all_system_messages(&mut self) {
    let mut message_list = self.system_drain(&LNIL);
    let mut error_msg: String = "".to_owned();
    while message_list.non_empty() && !self.is_closed() {
      let msg = message_list.head().clone().unwrap();
      message_list = message_list.tail();
      let mut msg_guard = msg.lock().unwrap();
      msg_guard.unlink();
      let inner_guard = self.inner.lock().unwrap();
      inner_guard
        .actor
        .as_ref()
        .iter()
        .for_each(|actor_cell_arc| {
          let mut actor_cell_guard = actor_cell_arc.lock().unwrap();
          actor_cell_guard.system_invoke(*msg_guard);
        });
      if inner_guard.terminate.load(Ordering::Relaxed) {
        error_msg = "Interrupted while processing system messages".to_owned();
      }
      if message_list.is_empty() && !self.is_closed() {
        message_list = self.system_drain(&LNIL);
      }
    }
    let dlm = if message_list.non_empty() {
      let inner_gurad = self.inner.lock().unwrap();
      inner_gurad.actor.as_ref().and_then(|actor_cell_arc| {
        let mut actor_cell_guard = actor_cell_arc.lock().unwrap();
        Some(actor_cell_guard.dead_letter_mailbox())
      })
    } else {
      None
    };
    while message_list.non_empty() {
      let msg = message_list.head().clone().unwrap();
      message_list = message_list.tail();
      let mut msg_guard = msg.lock().unwrap();
      msg_guard.unlink();
      dlm.iter().for_each(|e| {
        let mut dlm_guard = e.lock().unwrap();
        let inner_gurad = self.inner.lock().unwrap();
        let actor_arc = inner_gurad.actor.as_ref().unwrap();
        let actor_cell_guard = actor_arc.lock().unwrap();
        let my_self = &*actor_cell_guard.my_self();
        dlm_guard.system_enqueue(my_self, *msg_guard)
      });
    }
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

impl SystemMessageQueue for DefaultMailbox {
  fn system_enqueue(&mut self, receiver: &dyn ActorRef, mut message: SystemMessage) {
    let current_list = self.system_queue_get();
    let head_arc_opt = current_list.head();
    if head_arc_opt.iter().any(|head_arc| {
      let system_message_guard = head_arc.lock().unwrap();
      system_message_guard.is_no_message()
    }) {
      let mailbox_inner = self.inner.lock().unwrap();
      match mailbox_inner.actor.as_ref() {
        Some(actor_arc) => {
          let actor_cell_guard = actor_arc.lock().unwrap();
          let dead_letter_mailbox_arc = actor_cell_guard.dead_letter_mailbox();
          let mut system_message_queue_guard = dead_letter_mailbox_arc.lock().unwrap();
          system_message_queue_guard.system_enqueue(receiver, message.clone());
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
        // putに失敗した場合、やり直すが、実際には発生しない
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
    let head_arc_opt = current_list.head();
    if head_arc_opt.iter().any(|head_arc| {
      let system_message_guard = head_arc.lock().unwrap();
      system_message_guard.is_no_message()
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
    let head_arc_opt = current_list.head();
    head_arc_opt
      .map(|head_arc| {
        let system_message_guard = head_arc.lock().unwrap();
        match &*system_message_guard {
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
  use crate::kernel::{DummyActorRef, DummyActorCell};
  use crate::kernel::system_message::LNIL;
  use crate::kernel::system_message::SystemMessage::Create;

  #[test]
  fn system_enqueue() {
    init_logger();
    let dead_letter_queue: Arc<Mutex<VecQueue<Envelope>>> =
      Arc::new(Mutex::new(VecQueue::<Envelope>::new()));
    let dead_letter_mailbox = DefaultMailbox::new(dead_letter_queue);
    let actor_cell = DummyActorCell::new(Arc::new(Mutex::new(dead_letter_mailbox)));
    let queue = Arc::new(Mutex::new(VecQueue::<Envelope>::new()));

    let mut mailbox = DefaultMailbox::new(queue);
    mailbox.set_actor(Some(Arc::new(Mutex::new(actor_cell))));

    let dmmy_actor_ref = DummyActorRef;
    let system_message1 = SystemMessage::of_create(None);
    mailbox.system_enqueue(&dmmy_actor_ref, system_message1.clone());
    let system_message = SystemMessage::of_resume(None);
    mailbox.system_enqueue(&dmmy_actor_ref, system_message);

    assert!(mailbox.has_system_messages());

    let l = mailbox.system_drain(&LNIL);
    println!("{:?}", l);
    println!("{:?}", l.head());

    let head_g = l.head().unwrap().lock().unwrap();
    assert!(match (&*head_g, system_message1) {
      (Create { .. }, Create { .. }) => true,
      _ => false,
    })
  }

  #[test]
  fn has_system_messages() {
    init_logger();
    let vec_queue = VecQueue::<Envelope>::new();
    let queue = Arc::new(Mutex::new(vec_queue));
    let dead_letter_mailbox = DefaultMailbox::new(queue);
    assert_eq!(dead_letter_mailbox.has_system_messages(), false);
  }
}
