use anyhow::anyhow;
use anyhow::Result;

use crate::kernel::{ActorRef, Envelope};
use actuator_support_rs::collections::Queue;

pub enum MessageSize {
  Limit(usize),
  Limitless,
}

pub trait MessageQueue {
  type Item;
  fn enqueue(&mut self, handle: Self::Item) -> Result<()>;
  fn dequeue(&mut self) -> Result<Self::Item>;
  fn dequeue_as_option(&mut self) -> Option<Self::Item> {
    self.dequeue().ok()
  }
  fn number_of_messages(&self) -> MessageSize;

  fn has_messages(&self) -> bool;

  fn non_empty(&self) -> bool {
    self.has_messages()
  }

  fn is_empty(&self) -> bool {
    !self.non_empty()
  }
}

pub trait EnvelopeQueue: MessageQueue<Item = Envelope> {
  fn enqueue_with_receiver(&mut self, receiver: &dyn ActorRef, handle: Envelope) -> Result<()>;

  fn clean_up(
    &mut self,
    owner: &dyn ActorRef,
    dead_letters: &mut dyn EnvelopeQueue,
  );
}

pub struct VecQueue<E> {
  q: actuator_support_rs::collections::BlockingVecQueue<E>,
}

impl MessageQueue for VecQueue<Envelope> {
  type Item = Envelope;

  fn enqueue(&mut self, handle: Envelope) -> Result<()> {
    let _ = self.q.offer(handle)?;
    Ok(())
  }

  fn dequeue(&mut self) -> Result<Envelope> {
    self
      .q
      .poll()
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
  fn enqueue_with_receiver(&mut self, receiver: &dyn ActorRef, handle: Envelope) -> Result<()> {
    self.enqueue(handle)
  }

  fn clean_up(
    &mut self,
    owner: &dyn ActorRef,
    dead_letters: &mut dyn EnvelopeQueue,
  ) {
    if self.has_messages() {
      let mut envelope_result = self.dequeue();
      while let Ok(envelope) = &envelope_result {
        dead_letters.enqueue_with_receiver(owner, envelope.clone());
        envelope_result = self.dequeue();
      }
    }
  }
}
