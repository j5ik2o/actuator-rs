mod queue;

use std::sync::Arc;

use anyhow::Result;
use crate::kernel::{ActorRef, Envelope};
use std::time::Duration;
use std::collections::VecDeque;

pub trait Queue<T> {
  fn number_of_messages(&self) -> usize;
  fn has_messages(&self) -> bool;
  fn enqueue(&mut self, handle: T);
  fn dequeue(&mut self) -> Result<T>;
}

pub trait MessageQueue {
  fn enqueue(&mut self, receiver: Arc<dyn ActorRef>, handle: Envelope);
  fn dequeue(&mut self) -> Result<Envelope>;
  fn dequeue_as_option(&mut self) -> Option<Envelope> {
    self.dequeue().ok()
  }
  fn number_of_messages(&self) -> usize;
  fn has_messages(&self) -> bool;
  fn clean_up(&mut self, owner: Arc<dyn ActorRef>, dead_letters: &mut dyn MessageQueue);
}

pub trait QueueBasedMessageQueue: MessageQueue {
  fn queue(&self) -> &dyn Queue<Envelope>;
  fn queue_mut(&mut self) -> &mut dyn Queue<Envelope>;

  fn number_of_messages(&self) -> usize {
    self.queue().number_of_messages()
  }

  fn has_messages(&self) -> bool {
    !self.queue().has_messages()
  }

  fn clean_up(&mut self, owner: Arc<dyn ActorRef>, dead_letters: &mut dyn MessageQueue) {
    if QueueBasedMessageQueue::has_messages(self) {
      let mut envelope_result = self.dequeue();
      while let Ok(envelope) = &envelope_result {
        dead_letters.enqueue(owner.clone(), envelope.clone());
        envelope_result = self.dequeue();
      }
    }
  }
}

pub trait UnboundedQueueBasedMessageQueue: QueueBasedMessageQueue {
  fn enqueue(&mut self, _: Arc<dyn ActorRef>, handle: Envelope) {
    self.queue_mut().enqueue(handle);
  }
  fn dequeue(&mut self) -> Result<Envelope> {
    self.queue_mut().dequeue()
  }
}

pub trait BoundedQueueBasedMessageQueue: QueueBasedMessageQueue {
  fn push_timeout(&self) -> Duration;
  fn enqueue(&mut self, _: Arc<dyn ActorRef>, handle: Envelope) {
    let q = self.queue_mut();
    q.enqueue(handle);
  }
  fn dequeue(&mut self) -> Result<Envelope> {
    self.queue_mut().dequeue()
  }
}

pub trait Mailbox {
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
}
