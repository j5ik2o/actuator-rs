use crate::actor::actor_cell::actor_cell_ref::ActorCellRef;
use crate::actor::actor_cell::ActorCellBehavior;
use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::dispatch::envelope::Envelope;

use crate::dispatch::mailbox::mailbox_status::MailboxStatus;
use crate::dispatch::mailbox::mailbox_type::MailboxType;
use crate::dispatch::mailbox::{MailboxBehavior, MailboxInternal, Terminate};
use crate::dispatch::message::Message;
use crate::dispatch::message_queue::{MessageQueue, MessageQueueBehavior, MessageQueueSize};
use crate::dispatch::system_message::{
  EarliestFirstSystemMessageList, LatestFirstSystemMessageList, SystemMessage, SystemMessageList,
  SystemMessageQueueBehavior, LNIL,
};
use anyhow::Result;
use std::cmp::max;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct Mailbox<Msg: Message> {
  mailbox_type: MailboxType,
  current_status: Arc<AtomicU32>,
  actor_cell_opt: Option<ActorCellRef<Msg>>,
  message_queue: Arc<Mutex<MessageQueue<Msg>>>,
  system_message_opt: Arc<Mutex<Option<SystemMessage>>>,
  throughput: usize,
  is_throughput_deadline_time_defined: Arc<AtomicBool>,
  throughput_deadline_time: Duration,
  terminate: Arc<Mutex<Terminate>>,
}

unsafe impl<Msg: Message> Send for Mailbox<Msg> {}
unsafe impl<Msg: Message> Sync for Mailbox<Msg> {}

impl<Msg: Message> PartialEq for Mailbox<Msg> {
  fn eq(&self, other: &Self) -> bool {
    self.mailbox_type == other.mailbox_type
      && Arc::ptr_eq(&self.current_status, &other.current_status)
      && self.actor_cell_opt == other.actor_cell_opt
      && Arc::ptr_eq(&self.message_queue, &other.message_queue)
      && Arc::ptr_eq(&self.system_message_opt, &other.system_message_opt)
      && self.throughput == other.throughput
      && Arc::ptr_eq(
        &self.is_throughput_deadline_time_defined,
        &other.is_throughput_deadline_time_defined,
      )
      && self.throughput_deadline_time == other.throughput_deadline_time
      && Arc::ptr_eq(&self.terminate, &other.terminate)
  }
}

impl<Msg: Message> Mailbox<Msg> {
  pub fn new_with_message_queue(mailbox_type: MailboxType, message_queue: Arc<Mutex<MessageQueue<Msg>>>) -> Self {
    Self {
      mailbox_type,
      current_status: Arc::new(AtomicU32::new(MailboxStatus::Open as u32)),
      actor_cell_opt: None,
      message_queue,
      system_message_opt: Arc::new(Mutex::new(None)),
      throughput: 1,
      is_throughput_deadline_time_defined: Arc::new(AtomicBool::new(false)),
      throughput_deadline_time: Duration::from_millis(100),
      terminate: Arc::new(Mutex::new(Terminate::new())),
    }
  }

  pub(crate) fn _update_status(&self, old: u32, new: u32) -> bool {
    let current_status = self.current_status.load(Ordering::Relaxed);
    log::debug!("current_status = {}, old = {}, new = {}", current_status, old, new);
    if current_status == old {
      self.current_status.store(new, Ordering::Relaxed);
      true
    } else {
      false
    }
  }

  pub(crate) fn _set_status(&self, value: u32) {
    self.current_status.store(value, Ordering::Relaxed);
  }
}

#[async_trait::async_trait]
impl<Msg: Message> MailboxInternal<Msg> for Mailbox<Msg> {
  fn mailbox_type(&self) -> MailboxType {
    self.mailbox_type.clone()
  }

  fn set_actor_cell(&mut self, actor_cell: ActorCellRef<Msg>) {
    self.actor_cell_opt = Some(actor_cell);
  }

  fn get_actor_cell(&self) -> ActorCellRef<Msg> {
    self.actor_cell_opt.as_ref().unwrap().clone()
  }

  fn get_status(&self) -> MailboxStatus {
    let status = self.current_status.load(Ordering::Relaxed);
    MailboxStatus::try_from(status).unwrap()
  }

  fn is_throughput_deadline_time_defined(&self) -> bool {
    self.is_throughput_deadline_time_defined.load(Ordering::Relaxed)
  }

  fn should_process_message(&self) -> bool {
    let current_status = self.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::ShouldNotProcessMask as u32) == 0
  }

  fn is_suspend(&self) -> bool {
    let current_status = self.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::SuspendMask as u32) != 0
  }

  fn is_closed(&self) -> bool {
    let current_status = self.current_status.load(Ordering::Relaxed);
    current_status == MailboxStatus::Closed as u32
  }

  fn is_scheduled(&self) -> bool {
    let current_status = self.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::Scheduled as u32) != 0
  }

  fn can_be_scheduled_for_panic(&self, has_message_hint: bool, has_system_message_hint: bool) -> bool {
    let current_status = self.current_status.load(Ordering::Relaxed);
    match current_status {
      cs if cs == MailboxStatus::Open as u32 || cs == MailboxStatus::Scheduled as u32 => {
        has_message_hint || has_system_message_hint || self.has_messages()
      }
      cs if cs == MailboxStatus::Closed as u32 => false,
      _ => has_system_message_hint || self.has_system_messages(),
    }
  }

  fn suspend_count(&self) -> u32 {
    let current_status = self.current_status.load(Ordering::Relaxed);
    log::debug!(
      "current_status = {}, suspend_mask = {}",
      current_status,
      MailboxStatus::SuspendMask as u32
    );
    current_status / MailboxStatus::SuspendUnit as u32
  }

  fn set_as_scheduled(&mut self) -> bool {
    loop {
      let current_status = self.current_status.load(Ordering::Relaxed);
      let s = current_status;
      if (s & MailboxStatus::ShouldScheduleMask as u32) != MailboxStatus::Open as u32 {
        log::debug!("set_as_scheduled: false");
        return false;
      }
      if self._update_status(s, s | MailboxStatus::Scheduled as u32) {
        log::debug!("set_as_scheduled: true");
        return true;
      }
    }
  }

  fn set_as_idle(&mut self) -> bool {
    loop {
      let current_status = self.current_status.load(Ordering::Relaxed);
      let s = current_status;
      if self._update_status(s, s & !(MailboxStatus::Scheduled as u32)) {
        return true;
      }
    }
  }

  fn resume(&mut self) -> bool {
    loop {
      let current_status = self.current_status.load(Ordering::Relaxed);
      if current_status == MailboxStatus::Closed as u32 {
        self._set_status(MailboxStatus::Closed as u32);
        return false;
      }
      let s = current_status;
      let next = if s < MailboxStatus::SuspendUnit as u32 {
        s
      } else {
        s - MailboxStatus::SuspendUnit as u32
      };
      if self._update_status(s, next) {
        return next < MailboxStatus::SuspendUnit as u32;
      }
    }
  }

  fn suspend(&mut self) -> bool {
    loop {
      let current_status = self.current_status.load(Ordering::Relaxed);
      if current_status == MailboxStatus::Closed as u32 {
        self._set_status(MailboxStatus::Closed as u32);
        log::debug!("Closed: suspend: false");
        return false;
      }
      let s = current_status;
      if self._update_status(s, s + MailboxStatus::SuspendUnit as u32) {
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

  fn become_closed(&mut self) -> bool {
    loop {
      let current_status = self.current_status.load(Ordering::Relaxed);
      if current_status == MailboxStatus::Closed as u32 {
        self._set_status(MailboxStatus::Closed as u32);
        log::debug!("become_closed: false");
        return false;
      }
      let s = current_status;
      if self._update_status(s, MailboxStatus::Closed as u32) {
        log::debug!("become_closed: true");
        return true;
      }
    }
  }

  async fn process_mailbox_with(&mut self, mut left: usize, deadline_ns: u128) {
    while left > 0 {
      log::debug!("left = {}, deadline_ns = {}", left, deadline_ns);
      let is_should_process_message = self.should_process_message();
      log::debug!("should_process_message = {}", is_should_process_message);

      if !is_should_process_message {
        break;
      }

      log::debug!("dequeue start");
      let message = self.dequeue();

      match message {
        Ok(next) => {
          log::debug!("dequeue finished: {:?}", next);
          let is_throughput_deadline_time_defined = self.is_throughput_deadline_time_defined();
          log::debug!(
            "is_throughput_deadline_time_defined = {}",
            is_throughput_deadline_time_defined
          );
          {
            let actor_cell_guard = self.actor_cell_opt.as_mut().unwrap();
            actor_cell_guard.invoke(next.clone());
          }
          self.process_all_system_messages().await;
          let now = SystemTime::now();
          if is_throughput_deadline_time_defined && (now.elapsed().unwrap().as_nanos() as u128) >= deadline_ns {
            break;
          }
        }
        Err(err) => {
          log::error!("dequeue finished: {:?}", err);
          break;
        }
      }
      left -= 1;
    }
  }

  fn system_queue_get(&self) -> LatestFirstSystemMessageList {
    let system_message_opt_guard = self.system_message_opt.lock().unwrap();
    LatestFirstSystemMessageList::new(system_message_opt_guard.clone())
  }

  fn system_queue_put(&mut self, old: &LatestFirstSystemMessageList, new: &LatestFirstSystemMessageList) -> bool {
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

    let mut system_message_opt_guard = self.system_message_opt.lock().unwrap();
    let result = match new.head() {
      Some(arc) => {
        let system_message_guard = arc.lock().unwrap();
        *system_message_opt_guard = Some(system_message_guard.clone());
        true
      }
      None => {
        *system_message_opt_guard = None;
        true
      }
    };
    result
  }
}

#[async_trait::async_trait]
impl<Msg: Message> MailboxBehavior<Msg> for Mailbox<Msg> {
  fn number_of_messages(&self) -> MessageQueueSize {
    let message_queue_guard = self.message_queue.lock().unwrap();
    message_queue_guard.number_of_messages()
  }

  fn has_messages(&self) -> bool {
    let message_queue_guard = self.message_queue.lock().unwrap();
    message_queue_guard.has_messages()
  }

  fn enqueue(&mut self, receiver: ActorRefRef<Msg>, msg: Envelope) -> Result<()> {
    let mut message_queue_guard = self.message_queue.lock().unwrap();
    message_queue_guard.enqueue(receiver, msg)
  }

  fn dequeue(&mut self) -> Result<Envelope> {
    let mut message_queue_guard = self.message_queue.lock().unwrap();
    message_queue_guard.dequeue().map(|msg| {
      log::debug!("dequeue finished: {:?}", msg);
      msg.unwrap()
    })
  }

  async fn process_all_system_messages(&mut self) {
    let mut message_list = self.system_drain(&LNIL);
    log::debug!("message_list = {:?}", message_list);
    let mut error_msg = None;
    while message_list.non_empty() && !self.is_closed() {
      message_list = {
        let (head, tail) = message_list.head_with_tail().unwrap().clone();
        let mut msg = head.lock().unwrap();
        msg.unlink();
        let actor_cell_ref = self.actor_cell_opt.as_mut().unwrap();
        actor_cell_ref.system_invoke(&*msg);
        let terminate_guard = self.terminate.lock().unwrap();
        if terminate_guard.get() {
          error_msg = Some("Interrupted while processing system messages".to_owned());
        }
        tail
      };
      if message_list.is_empty() && !self.is_closed() {
        message_list = self.system_drain(&LNIL);
      }
    }
    let dead_letter_mailbox = if message_list.non_empty() {
      self
        .actor_cell_opt
        .as_ref()
        .and_then(|actor_cell| Some(actor_cell.dead_letter_mailbox()))
    } else {
      None
    };
    while message_list.non_empty() {
      {
        let system_message = message_list.head().clone().unwrap();
        let mut system_message_guard = system_message.lock().unwrap();
        system_message_guard.unlink();
        dead_letter_mailbox.iter().for_each(|e| {
          let mut dead_letter_mailbox_guard = e.lock().unwrap();
          let actor_cell = self.actor_cell_opt.as_ref().unwrap();
          let my_self = actor_cell.actor_ref_ref().to_any_ref_ref();
          dead_letter_mailbox_guard.system_enqueue(my_self.clone(), &mut system_message_guard);
        });
      }
      message_list = message_list.tail();
    }
    if error_msg.is_some() {
      panic!("{}", error_msg.unwrap())
    }
  }

  async fn process_mailbox(&mut self) {
    let (left, deadline_ns) = {
      let l = max(self.throughput, 1);
      let d = if self.is_throughput_deadline_time_defined.load(Ordering::Relaxed) {
        let now = SystemTime::now();
        now.elapsed().unwrap().as_nanos() + self.throughput_deadline_time.as_nanos()
      } else {
        0
      };
      (l, d)
    };
    self.process_mailbox_with(left, deadline_ns).await
  }

  async fn execute(&mut self) {
    log::debug!("execute: start");
    if !self.is_closed() {
      log::debug!("execute: self.process_all_system_messages()");
      self.process_all_system_messages().await;
      log::debug!("execute: self.process_mailbox()");
      self.process_mailbox().await;
    }
    log::debug!("execute: self.set_as_idle()");
    self.set_as_idle();
    let actor_cell_ref = self.actor_cell_opt.as_ref().unwrap();
    let mut dispatcher_ref = actor_cell_ref.dispatcher();
    log::debug!("execute: dispatcher_ref.register_for_execution(self.clone(), false, false)");
    dispatcher_ref.register_for_execution(self.clone(), false, false);
  }
}
impl<Msg: Message> SystemMessageQueueBehavior<Msg> for Mailbox<Msg> {
  fn system_enqueue(&mut self, receiver: ActorRefRef<Msg>, message: &mut SystemMessage) {
    let current_list = self.system_queue_get();
    let head_arc_opt = current_list.head();
    if head_arc_opt.iter().any(|head_arc| {
      let system_message_guard = head_arc.lock().unwrap();
      system_message_guard.is_no_message()
    }) {
      match &self.actor_cell_opt {
        Some(actor_cell) => {
          let dead_letter_mailbox = actor_cell.dead_letter_mailbox();
          let mut dead_letter_mailbox_guard = dead_letter_mailbox.lock().unwrap();
          let wrapper_ref_ref = receiver.clone().to_any_ref_ref();
          dead_letter_mailbox_guard.system_enqueue(wrapper_ref_ref, message);
        }
        None => {
          log::warn!("The message was not delivered to the dead letter.");
        }
      }
    } else {
      if !self.system_queue_put(&current_list.clone(), &current_list.prepend(message.clone())) {
        // putに失敗した場合、やり直すが、実際には発生しない
        message.unlink();
        self.system_enqueue(receiver, message);
      }
    }
  }

  fn system_drain(&mut self, new_contents: &LatestFirstSystemMessageList) -> EarliestFirstSystemMessageList {
    loop {
      let current_list = self.system_queue_get();
      let head_arc_opt = current_list.head();
      if head_arc_opt.iter().any(|head_arc| {
        let system_message_guard = head_arc.lock().unwrap();
        system_message_guard.is_no_message()
      }) {
        return EarliestFirstSystemMessageList::new(None);
      } else if self.system_queue_put(&current_list, new_contents) {
        return current_list.reverse();
      }
    }
  }

  fn has_system_messages(&self) -> bool {
    let current_list = self.system_queue_get();
    let head_arc_opt = current_list.head();
    head_arc_opt
      .map(|head_arc| {
        let system_message_guard = head_arc.lock().unwrap();
        match &*system_message_guard {
          SystemMessage::NoMessage { .. } => false,
          _ => true,
        }
      })
      .unwrap_or(false)
  }
}
