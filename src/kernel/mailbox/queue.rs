use crate::kernel::mailbox::Queue;
use crate::kernel::Envelope;
use std::collections::VecDeque;
use anyhow::Result;
use anyhow::anyhow;

impl Queue<Envelope> for VecDeque<Envelope> {
  fn number_of_messages(&self) -> usize {
    self.len()
  }

  fn has_messages(&self) -> bool {
    !self.is_empty()
  }

  fn enqueue(&mut self, handle: Envelope) {
    self.push_back(handle);
  }

  fn dequeue(&mut self) -> Result<Envelope> {
    self
      .pop_front()
      .ok_or(Err(anyhow!("occurred error: no such element"))?)
  }
}
