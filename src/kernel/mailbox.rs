use async_trait::async_trait;
use crate::kernel::{ActorRef, Envelope};
use crate::kernel::mailbox::queue::{EnvelopeQueue, MessageSize};
use anyhow::Result;

mod queue;

#[async_trait]
pub trait Mailbox {
  fn actor(&self) -> &dyn ActorRef;
  fn queue(&self) -> &dyn EnvelopeQueue;

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

  fn process_all_system_messages();

  async fn run();
}

pub struct DefaultMailbox {}
