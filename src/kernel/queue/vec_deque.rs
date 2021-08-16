use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::kernel::{Message, QueueWriter, Envelope, QueueReader, MessageSize};

#[derive(Clone)]
pub struct QueueInVecQueue<M: Message> {
  inner: Arc<Mutex<QueueInVecQueueInner<M>>>,
}

struct QueueInVecQueueInner<M: Message> {
  queue: VecDeque<Envelope<M>>,
}

impl<M: Message> QueueInVecQueue<M> {
  pub fn new(queue: VecDeque<Envelope<M>>) -> Self {
    let inner = Arc::from(Mutex::new(QueueInVecQueueInner { queue }));
    Self { inner }
  }
}

impl<M: Message> QueueReader<M> for QueueInVecQueue<M> {
  fn dequeue(&self) -> Envelope<M> {
    let mut inner = self.inner.lock().unwrap();
    inner.queue.pop_back().unwrap()
  }

  fn try_dequeue(&self) -> anyhow::Result<Option<Envelope<M>>> {
    let mut inner = self.inner.lock().unwrap();
    let result = inner.queue.pop_back();
    Ok(result)
  }

  fn non_empty(&self) -> bool {
    let inner = self.inner.lock().unwrap();
    !inner.queue.is_empty()
  }

  fn number_of_messages(&self) -> MessageSize {
    let inner = self.inner.lock().unwrap();
    MessageSize::Limit(inner.queue.len())
  }
}

impl<M: Message> QueueWriter<M> for QueueInVecQueue<M> {
  fn try_enqueue(&self, msg: Envelope<M>) -> anyhow::Result<()> {
    let mut inner = self.inner.lock().unwrap();
    inner.queue.push_back(msg);
    Ok(())
  }
}
