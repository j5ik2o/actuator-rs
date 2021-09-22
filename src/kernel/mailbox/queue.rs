use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Result;

use crate::kernel::{ActorRef, Envelope};
use crate::kernel::mailbox::Queue;
use actuator_support_rs::collections::Queue;

pub enum MessageSize {
  Limit(usize),
  Limitless,
}

pub trait MessageQueue {
  type Item;
  fn enqueue(&mut self, handle: Self::Item) -> Result<()>;
  fn dequeue(&mut self) -> Result<Self::Item>;

  fn number_of_messages(&self) -> MessageSize;

  fn has_messages(&self) -> bool;

  fn non_empty(&self) -> bool {
    self.has_messages()
  }

  fn is_empty(&self) -> bool {
    !self.non_empty()
  }
}

pub trait EnvelopeQueue: MessageQueue {
  type Item = Envelope;
  fn enqueue(&mut self, receiver: &dyn ActorRef, handle: Envelope) -> Result<()>;

  fn dequeue(&mut self) -> Result<Envelope>;
  fn dequeue_as_option(&mut self) -> Option<Envelope> {
    self.dequeue().ok()
  }

  fn clean_up(&mut self, owner: &dyn ActorRef, dead_letters: &mut dyn MessageQueue);
}

pub struct VecQueue<E>{
 q: actuator_support_rs::collections::BlockingVecQueue<E>
}

impl MessageQueue for VecQueue<Envelope> {
  type Item = Envelope;

  fn enqueue(&mut self, handle: Envelope) -> Result<()> {
    let _ = self.q.offer(handle)?;
    Ok(())
  }

  fn dequeue(&mut self) -> Result<Envelope> {
    self.q.poll()
      .ok_or(Err(anyhow!("occurred error: no such element"))?)
  }

  fn number_of_messages(&self) -> MessageSize {
    MessageSize::Limit(self.q.len())
  }

  fn has_messages(&self) -> bool {
    self.q.len() > 0
  }

}

impl EnvelopeQueue for VecQueue<Envelope> {
  fn enqueue(&mut self, receiver: &dyn ActorRef, handle: Envelope) -> Result<()> {
    self.enqueue(handle)
  }

  fn dequeue(&mut self) -> Result<Envelope> {
    MessageQueue::dequeue(self)
  }

  fn clean_up(&mut self, owner: &dyn ActorRef, dead_letters: &mut dyn MessageQueue) {
    if self.has_messages() {
      let mut envelope_result = self.dequeue();
      while let Ok(envelope) = &envelope_result {
        dead_letters.enqueue(owner, envelope.clone());
        envelope_result = self.dequeue();
      }
    }
  }
}