use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::kernel::{Message, QueueWriter, Envelope, QueueReader};

#[derive(Clone)]
pub struct QueueReaderInVecQueue<M: Message> {
  inner: Arc<Mutex<QueueReaderInVecQueueInner<M>>>,
}

struct QueueReaderInVecQueueInner<M: Message> {
  queue: VecDeque<Envelope<M>>,
}

impl<M: Message> QueueReaderInVecQueue<M> {
    pub fn new(inner: Arc<VecDeque<Envelope<M>>>) -> Self {
        let q = Arc::try_unwrap(inner).unwrap();
        let inner = Arc::from(Mutex::new(QueueReaderInVecQueueInner { queue: q }));
        Self { inner }
    }
}

impl<M: Message> QueueReader<M> for QueueReaderInVecQueue<M> {
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

  fn number_of_messages(&self) -> usize {
    let inner = self.inner.lock().unwrap();
    inner.queue.len()
  }
}

