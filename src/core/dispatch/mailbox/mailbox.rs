use crate::core::actor::actor_ref::ActorRef;
use crate::core::dispatch::dispatcher::Dispatcher;
use crate::core::dispatch::envelope::Envelope;
use crate::core::dispatch::mailbox::mailbox_status::MailboxStatus;
use crate::core::dispatch::mailbox::mailbox_type::MailboxType;
use crate::core::dispatch::mailbox::system_mailbox::SystemMailbox;
use crate::core::dispatch::mailbox::{MailboxBehavior, MailboxReaderBehavior, MailboxWriterBehavior};
use crate::core::dispatch::message::Message;
use crate::core::dispatch::message_queue::{
  MessageQueue, MessageQueueBehavior, MessageQueueReaderBehavior, MessageQueueReaderFactoryBehavior, MessageQueueSize,
  MessageQueueWriterBehavior, MessageQueueWriterFactoryBehavior,
};
use crate::core::dispatch::system_message::earliest_first_system_message_list::EarliestFirstSystemMessageList;
use crate::core::dispatch::system_message::latest_first_system_message_list::LatestFirstSystemMessageList;

use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::system_message::system_message_entry::SystemMessageEntry;
use crate::core::dispatch::system_message::{SystemMessageQueueReaderBehavior, SystemMessageQueueWriterBehavior};
use anyhow::Result;

use crate::core::actor::actor_cell_with_ref::ActorCellWithRef;
use crate::infrastructure::logging_mutex::LoggingMutex;

use crate::mutex_lock_with_log;
use std::cmp::max;
use std::fmt::{Debug, Formatter};

use crate::core::actor::actor_context::ActorContext;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct Terminate {
  value: AtomicBool,
}

impl Terminate {
  pub fn new() -> Self {
    Self {
      value: AtomicBool::new(false),
    }
  }

  pub fn set(&self, value: bool) {
    self.value.store(value, Ordering::Relaxed);
  }

  pub fn get(&self) -> bool {
    self.value.load(Ordering::Relaxed)
  }
}

#[derive(Debug)]
struct MailboxInner<Msg: Message> {
  mailbox_type: MailboxType,
  current_status: Arc<AtomicU32>,
  message_queue: MessageQueue<Msg>,
  system_mailbox: SystemMailbox<Msg>,
  throughput: usize,
  is_throughput_deadline_time_defined: Arc<AtomicBool>,
  throughput_deadline_time: Duration,
  terminate: Arc<Mutex<Terminate>>,
}

#[derive(Clone)]
pub struct Mailbox<Msg: Message> {
  inner: Arc<LoggingMutex<MailboxInner<Msg>>>,
}

impl<Msg: Message> Debug for Mailbox<Msg> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let inner = mutex_lock_with_log!(self.inner, "fmt");
    write!(f, "{:?}", inner)
  }
}

unsafe impl<Msg: Message> Send for Mailbox<Msg> {}
unsafe impl<Msg: Message> Sync for Mailbox<Msg> {}

impl<Msg: Message> PartialEq for Mailbox<Msg> {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.inner, &other.inner)
    // self.mailbox_type == other.mailbox_type
    //   && Arc::ptr_eq(&self.current_status, &other.current_status)
    //   && self.message_queue == other.message_queue
    //   && self.system_mailbox == other.system_mailbox
    //   && self.throughput == other.throughput
    //   && Arc::ptr_eq(
    //     &self.is_throughput_deadline_time_defined,
    //     &other.is_throughput_deadline_time_defined,
    //   )
    //   && self.throughput_deadline_time == other.throughput_deadline_time
    //   && Arc::ptr_eq(&self.terminate, &other.terminate)
  }
}

impl Mailbox<AnyMessage> {
  pub fn to_typed<Msg: Message>(self) -> Mailbox<Msg> {
    let inner = mutex_lock_with_log!(self.inner, "to_typed");
    Mailbox {
      inner: Arc::new(LoggingMutex::new(
        "Mailbox#inner",
        MailboxInner {
          mailbox_type: inner.mailbox_type.clone(),
          current_status: inner.current_status.clone(),
          message_queue: inner.message_queue.clone().to_typed(),
          system_mailbox: inner.system_mailbox.clone().to_typed(),
          throughput: inner.throughput,
          is_throughput_deadline_time_defined: inner.is_throughput_deadline_time_defined.clone(),
          throughput_deadline_time: inner.throughput_deadline_time,
          terminate: inner.terminate.clone(),
        },
      )),
    }
  }
}

impl<Msg: Message> Mailbox<Msg> {
  pub fn new_with_message_queue(mailbox_type: MailboxType, message_queue: MessageQueue<Msg>) -> Self {
    Self {
      inner: Arc::new(LoggingMutex::new(
        "Mailbox#inner",
        MailboxInner {
          mailbox_type,
          current_status: Arc::new(AtomicU32::new(MailboxStatus::Open as u32)),
          message_queue,
          system_mailbox: SystemMailbox::new(),
          throughput: 1,
          is_throughput_deadline_time_defined: Arc::new(AtomicBool::new(false)),
          throughput_deadline_time: Duration::from_millis(100),
          terminate: Arc::new(Mutex::new(Terminate::new())),
        },
      )),
    }
  }

  pub fn to_any(self) -> Mailbox<AnyMessage> {
    let inner = mutex_lock_with_log!(self.inner, "to_any");
    Mailbox {
      inner: Arc::new(LoggingMutex::new(
        "Mailbox#inner",
        MailboxInner {
          mailbox_type: inner.mailbox_type.clone(),
          current_status: inner.current_status.clone(),
          message_queue: inner.message_queue.clone().to_any(),
          system_mailbox: inner.system_mailbox.clone().to_any(),
          throughput: inner.throughput,
          is_throughput_deadline_time_defined: inner.is_throughput_deadline_time_defined.clone(),
          throughput_deadline_time: inner.throughput_deadline_time,
          terminate: inner.terminate.clone(),
        },
      )),
    }
  }

  pub fn sender(&self) -> MailboxSender<Msg> {
    MailboxSender {
      underlying: self.clone(),
    }
  }

  fn _update_status(&mut self, old: u32, new: u32) -> bool {
    let inner = mutex_lock_with_log!(self.inner, "_update_status");
    match inner
      .current_status
      .compare_exchange(old, new, Ordering::Relaxed, Ordering::Relaxed)
    {
      Ok(_) => true,
      Err(_) => false,
    }
    // let current_status = self.current_status.load(Ordering::Relaxed);
    // log::debug!("current_status = {}, old = {}, new = {}", current_status, old, new);
    // if current_status == old {
    //   self.current_status.store(new, Ordering::Relaxed);
    //   true
    // } else {
    //   false
    // }
  }

  fn _set_status(&mut self, value: u32) {
    let inner = mutex_lock_with_log!(self.inner, "_set_status");
    inner.current_status.store(value, Ordering::Relaxed);
  }

  pub fn get_status(&self) -> MailboxStatus {
    let inner = mutex_lock_with_log!(self.inner, "get_status");
    let status = inner.current_status.load(Ordering::Relaxed);
    MailboxStatus::try_from(status).unwrap()
  }

  pub fn is_throughput_deadline_time_defined(&self) -> bool {
    let inner = mutex_lock_with_log!(self.inner, "is_throughput_deadline_time_defined");
    inner.is_throughput_deadline_time_defined.load(Ordering::Relaxed)
  }

  pub fn should_process_message(&self) -> bool {
    let inner = mutex_lock_with_log!(self.inner, "should_process_message");
    let current_status = inner.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::ShouldNotProcessMask as u32) == 0
  }

  pub fn is_suspend(&self) -> bool {
    let inner = mutex_lock_with_log!(self.inner, "is_suspend");
    let current_status = inner.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::SuspendMask as u32) != 0
  }

  pub fn is_closed(&self) -> bool {
    let inner = mutex_lock_with_log!(self.inner, "is_closed");
    let current_status = inner.current_status.load(Ordering::Relaxed);
    current_status == MailboxStatus::Closed as u32
  }

  pub fn is_scheduled(&self) -> bool {
    let inner = mutex_lock_with_log!(self.inner, "is_scheduled");
    let current_status = inner.current_status.load(Ordering::Relaxed);
    (current_status & MailboxStatus::Scheduled as u32) != 0
  }

  pub fn can_be_scheduled_for_panic(&self, has_message_hint: bool, has_system_message_hint: bool) -> bool {
    let current_status = {
      let inner = mutex_lock_with_log!(self.inner, "is_scheduled");
      inner.current_status.load(Ordering::Relaxed)
    };
    match current_status {
      cs if cs == MailboxStatus::Open as u32 || cs == MailboxStatus::Scheduled as u32 => {
        has_message_hint || has_system_message_hint || self.has_messages()
      }
      cs if cs == MailboxStatus::Closed as u32 => false,
      _ => {
        has_system_message_hint || {
          let inner = mutex_lock_with_log!(self.inner, "can_be_scheduled_for_panic");
          inner.system_mailbox.has_system_messages()
        }
      }
    }
  }

  pub fn suspend_count(&self) -> u32 {
    let inner = mutex_lock_with_log!(self.inner, "is_scheduled");
    let current_status = inner.current_status.load(Ordering::Relaxed);
    log::debug!(
      "current_status = {}, suspend_mask = {}",
      current_status,
      MailboxStatus::SuspendMask as u32
    );
    current_status / MailboxStatus::SuspendUnit as u32
  }

  pub fn set_as_scheduled(&mut self) -> bool {
    loop {
      let current_status = {
        let inner = mutex_lock_with_log!(self.inner, "is_scheduled");
        inner.current_status.load(Ordering::Relaxed)
      };
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

  pub fn set_as_idle(&mut self) -> bool {
    loop {
      let current_status = {
        let inner = mutex_lock_with_log!(self.inner, "set_as_idle");
        inner.current_status.load(Ordering::Relaxed)
      };
      let s = current_status;
      if self._update_status(s, s & !(MailboxStatus::Scheduled as u32)) {
        return true;
      }
    }
  }

  pub fn resume(&mut self) -> bool {
    loop {
      let current_status = {
        let inner = mutex_lock_with_log!(self.inner, "resume");
        inner.current_status.load(Ordering::Relaxed)
      };
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

  pub fn suspend(&mut self) -> bool {
    loop {
      let current_status = {
        let inner = mutex_lock_with_log!(self.inner, "suspend");
        inner.current_status.load(Ordering::Relaxed)
      };
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

  pub fn become_closed(&mut self) -> bool {
    loop {
      let current_status = {
        let inner = mutex_lock_with_log!(self.inner, "suspend");
        inner.current_status.load(Ordering::Relaxed)
      };
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

  async fn process_mailbox(&mut self, actor_cell: ActorCellWithRef<Msg>) {
    let (left, deadline_ns) = {
      let throughput = {
        let inner = mutex_lock_with_log!(self.inner, "process_mailbox");
        inner.throughput
      };
      let l = max(throughput, 1);
      let is_throughput_deadline_time_defined = {
        let inner = mutex_lock_with_log!(self.inner, "process_mailbox");
        inner.is_throughput_deadline_time_defined.clone()
      };
      let d = if is_throughput_deadline_time_defined.load(Ordering::Relaxed) {
        let now = SystemTime::now();
        let throughput_deadline_time = {
          let inner = mutex_lock_with_log!(self.inner, "process_mailbox");
          inner.throughput_deadline_time
        };
        now.elapsed().unwrap().as_nanos() + throughput_deadline_time.as_nanos()
      } else {
        0
      };
      (l, d)
    };
    self.process_mailbox_with(left, deadline_ns, actor_cell).await
  }

  async fn process_mailbox_with(&mut self, mut left: usize, deadline_ns: u128, mut actor_cell: ActorCellWithRef<Msg>) {
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
        Ok(Some(next)) => {
          log::debug!("dequeue finished: {:?}", next);
          let is_throughput_deadline_time_defined = self.is_throughput_deadline_time_defined();
          log::debug!(
            "is_throughput_deadline_time_defined = {}",
            is_throughput_deadline_time_defined
          );
          actor_cell.invoke(&next);
          self.process_system_mailbox(actor_cell.clone(), self.clone()).await;
          let now = SystemTime::now();
          if is_throughput_deadline_time_defined && (now.elapsed().unwrap().as_nanos() as u128) >= deadline_ns {
            break;
          }
        }
        Ok(None) => {
          log::debug!("dequeue finished: None")
        }
        Err(err) => {
          log::error!("dequeue finished: {:?}", err);
          break;
        }
      }
      left -= 1;
    }
  }

  async fn process_system_mailbox(&mut self, actor_cell: ActorCellWithRef<Msg>, mailbox: Mailbox<Msg>) {
    let mut system_mailbox = {
      let inner = mutex_lock_with_log!(self.inner, "process_system_mailbox");
      inner.system_mailbox.clone()
    };
    system_mailbox
      .process_all_system_messages(
        || mailbox.is_closed(),
        || {
          let terminate = {
            let inner = mutex_lock_with_log!(self.inner, "process_system_mailbox");
            inner.terminate.clone()
          };
          let t = terminate.lock().unwrap();
          t.get()
        },
        actor_cell.clone(),
      )
      .await;
  }
}

#[derive(Debug, Clone)]
pub struct MailboxSender<Msg: Message> {
  underlying: Mailbox<Msg>,
}

impl MailboxSender<AnyMessage> {
  pub fn to_typed<Msg: Message>(self) -> MailboxSender<Msg> {
    MailboxSender {
      underlying: self.underlying.to_typed(),
    }
  }
}

impl<Msg: Message> MailboxSender<Msg> {
  pub fn to_any(self) -> MailboxSender<AnyMessage> {
    MailboxSender {
      underlying: self.underlying.to_any(),
    }
  }
}

impl<Msg: Message> MailboxBehavior<Msg> for Mailbox<Msg> {
  fn number_of_messages(&self) -> MessageQueueSize {
    let inner = mutex_lock_with_log!(self.inner, "number_of_messages");
    inner.message_queue.number_of_messages()
  }

  fn has_messages(&self) -> bool {
    let inner = mutex_lock_with_log!(self.inner, "number_of_messages");
    inner.message_queue.has_messages()
  }
}

impl<Msg: Message> SystemMessageQueueWriterBehavior<Msg> for MailboxSender<Msg> {
  fn system_enqueue(&mut self, receiver: ActorRef<Msg>, message: &mut SystemMessageEntry) {
    let system_mailbox = {
      let inner = mutex_lock_with_log!(self.underlying.inner, "system_enqueue");
      inner.system_mailbox.clone()
    };
    system_mailbox.sender().system_enqueue(receiver, message)
  }
}

impl<Msg: Message> SystemMessageQueueReaderBehavior for Mailbox<Msg> {
  fn has_system_messages(&self) -> bool {
    let inner = mutex_lock_with_log!(self.inner, "has_system_messages");
    inner.system_mailbox.has_system_messages()
  }

  fn system_drain(&mut self, new_contents: &LatestFirstSystemMessageList) -> EarliestFirstSystemMessageList {
    let mut inner = mutex_lock_with_log!(self.inner, "system_drain");
    inner.system_mailbox.system_drain(new_contents)
  }
}

#[async_trait::async_trait]
impl<Msg: Message> MailboxReaderBehavior<Msg> for Mailbox<Msg> {
  fn dequeue(&mut self) -> Result<Option<Envelope>> {
    let mut qr = {
      let inner = mutex_lock_with_log!(self.inner, "dequeue");
      inner.message_queue.reader()
    };
    qr.dequeue()
  }

  async fn execute(&mut self, actor_cell: ActorCellWithRef<Msg>, mut dispatcher: Dispatcher) {
    log::debug!("execute: start");
    if !self.is_closed() {
      log::debug!("execute: self.process_all_system_messages()");
      self.process_system_mailbox(actor_cell.clone(), self.clone()).await;
      log::debug!("execute: self.process_mailbox()");
      self.process_mailbox(actor_cell.clone()).await;
    }
    log::debug!("execute: self.set_as_idle()");
    self.set_as_idle();
    log::debug!("execute: register_for_execution");
    dispatcher.register_for_execution(actor_cell, false, false);
  }
}

impl<Msg: Message> MailboxBehavior<Msg> for MailboxSender<Msg> {
  fn number_of_messages(&self) -> MessageQueueSize {
    self.underlying.number_of_messages()
  }

  fn has_messages(&self) -> bool {
    self.underlying.has_messages()
  }
}

impl<Msg: Message> MailboxWriterBehavior<Msg> for MailboxSender<Msg> {
  fn enqueue(&mut self, receiver: ActorRef<Msg>, msg: Envelope) -> Result<()> {
    let mq = {
      let inner = mutex_lock_with_log!(self.underlying.inner, "enqueue");
      inner.message_queue.clone()
    };
    mq.writer().enqueue(receiver, msg)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::dispatch::mailbox::mailbox_type::MailboxTypeBehavior;
  use crate::core::dispatch::system_message::system_message::SystemMessage;
  use crate::core::dispatch::system_message::LNIL;
  use std::env;

  fn init_logger() {
    env::set_var("RUST_LOG", "debug");
    let _ = env_logger::builder().is_test(true).try_init();
  }

  #[test]
  fn test() {
    init_logger();
    let mailbox_type = MailboxType::of_unbounded();
    let mq: MessageQueue<String> = mailbox_type.create_message_queue(None);

    let mut m = Mailbox::new_with_message_queue(mailbox_type, mq);
    let mut ms = m.sender();

    ms.enqueue(ActorRef::NoSender, Envelope::new("".to_string())).unwrap();

    let sm = SystemMessage::of_create();
    ms.system_enqueue(ActorRef::NoSender, &mut SystemMessageEntry::new(sm.clone()));

    let result = m.dequeue().unwrap();
    assert_eq!(result.unwrap().typed_message::<String>().unwrap(), "".to_string());

    assert!(m.has_system_messages());
    let result = m.system_drain(&LNIL.clone());
    let sm_actual = result.head.unwrap().lock().unwrap().message.clone();
    assert_eq!(sm_actual, sm);
  }
}
