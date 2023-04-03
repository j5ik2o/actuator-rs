// see https://github.com/apache/incubator-pekko/blob/main/actor/src/main/scala/org/apache/pekko/dispatch/Mailbox.scala

pub mod mailbox;
pub mod mailbox_status;
pub mod mailbox_type;

use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;

use crate::actor::actor_cell::actor_cell_ref::{ActorCellRef, ExtendedCellRef};

use crate::actor::actor_ref::actor_ref_ref::ActorRefRef;
use crate::dispatch::envelope::Envelope;

use crate::dispatch::mailbox::mailbox_type::MailboxType;
use crate::dispatch::message::Message;
use crate::dispatch::message_queue::MessageQueueSize;
use crate::dispatch::system_message::LatestFirstSystemMessageList;
use mailbox_status::MailboxStatus;

#[async_trait::async_trait]
pub trait MailboxBehavior<Msg: Message> {
  fn number_of_messages(&self) -> MessageQueueSize;
  fn has_messages(&self) -> bool;
  fn enqueue(&mut self, receiver: ActorRefRef<Msg>, msg: Envelope) -> Result<()>;
  fn dequeue(&mut self) -> Result<Envelope>;
  async fn process_all_system_messages(&mut self);
  async fn process_mailbox(&mut self);
  async fn execute(&mut self);
}

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

#[async_trait::async_trait]
pub trait MailboxInternal<Msg: Message> {
  fn mailbox_type(&self) -> MailboxType;
  fn set_actor_cell(&mut self, actor_cell: ExtendedCellRef<Msg>);
  fn get_actor_cell(&self) -> ExtendedCellRef<Msg>;
  fn get_status(&self) -> MailboxStatus;
  fn is_throughput_deadline_time_defined(&self) -> bool;
  fn should_process_message(&self) -> bool;
  fn is_suspend(&self) -> bool;
  fn is_closed(&self) -> bool;
  fn is_scheduled(&self) -> bool;
  fn can_be_scheduled_for_panic(&self, has_message_hint: bool, has_system_message_hint: bool) -> bool;
  fn suspend_count(&self) -> u32;
  fn set_as_scheduled(&mut self) -> bool;
  fn set_as_idle(&mut self) -> bool;
  fn resume(&mut self) -> bool;
  fn suspend(&mut self) -> bool;
  fn become_closed(&mut self) -> bool;
  async fn process_mailbox_with(&mut self, left: usize, deadline_ns: u128);
  fn system_queue_get(&self) -> LatestFirstSystemMessageList;
  fn system_queue_put(&mut self, old: &LatestFirstSystemMessageList, new: &LatestFirstSystemMessageList) -> bool;
}
